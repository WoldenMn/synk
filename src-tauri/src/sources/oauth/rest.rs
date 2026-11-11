use std::sync::Arc;
use std::time::Duration;

use reqwest::{Method, StatusCode};
use serde::de::DeserializeOwned;
use sqlx::SqlitePool;
use zeroize::Zeroizing;

use super::brand::{Brand, Refresh, Reply, Screen, Throttle};
use super::clock::Clock;
use super::pacer::Pacer;
use super::refresh::refresh_locked;
use crate::crypto::CryptoVault;
use crate::error::AppError;
use crate::sources::{store, SourceId};
use crate::utils::http::client;
use crate::utils::redact::redact;
use crate::utils::text::truncate_chars;

pub const MAX_ATTEMPTS: u32 = 3;
pub const BASE_BACKOFF: Duration = Duration::from_secs(1);
pub const RETRY_AFTER_CAP: Duration = Duration::from_secs(60);

pub struct RestClient {
    http: reqwest::Client,
    brand: &'static Brand,
    token: Zeroizing<String>,
    user_ref: String,
    refresh: Option<Refresh>,
    refreshed: bool,
    clock: Option<Arc<dyn Clock>>,
    pacer: Option<Pacer>,
    quota_resume: Option<Duration>,
}

impl RestClient {
    pub async fn connect(
        brand: &'static Brand,
        source: SourceId,
        api_base: &str,
        pool: &SqlitePool,
        vault: &Arc<CryptoVault>,
        refresh: Option<Refresh>,
    ) -> Result<Self, AppError> {
        let stored = store::load_auth(pool, vault.as_ref(), source)
            .await?
            .ok_or_else(|| {
                AppError::Unauthorized(format!("No {} authentication found", brand.label))
            })?;
        let mut token = stored.access_token.clone();
        if let Some(r) = &refresh {
            if stored.expires_at < chrono::Utc::now().timestamp() + 300 {
                token = match refresh_locked(r.spec, &r.base, pool, vault.as_ref(), &token).await {
                    Ok(fresh) => fresh,
                    Err(e) => {
                        return Err(match r.failure {
                            Some(f) => f(e),
                            None => e,
                        })
                    }
                };
            }
        }
        Ok(Self {
            http: client(api_base, Duration::from_secs(30))?,
            brand,
            token,
            user_ref: stored.user_ref,
            refresh,
            refreshed: false,
            clock: None,
            pacer: None,
            quota_resume: None,
        })
    }

    pub fn paced(mut self, clock: Arc<dyn Clock>, per_minute: usize) -> Self {
        self.pacer = Some(Pacer::new(clock.clone(), per_minute));
        self.clock = Some(clock);
        self
    }

    pub fn user_ref(&self) -> &str {
        &self.user_ref
    }

    pub fn quota_resume(&self) -> Option<Duration> {
        self.quota_resume
    }

    async fn pause(&self, wait: Duration) {
        match &self.clock {
            Some(clock) => clock.sleep(wait).await,
            None => tokio::time::sleep(wait).await,
        }
    }

    pub fn decode<T: DeserializeOwned>(&self, text: &str) -> Result<T, AppError> {
        serde_json::from_str(text).map_err(|e| {
            log::debug!(
                "[{}] reponse non decodee : {}",
                self.brand.tag,
                truncate_chars(&redact(text), 300)
            );
            AppError::Internal(format!(
                "{} : reponse illisible ({} octets) : {}",
                self.brand.label,
                text.len(),
                redact(&e.to_string())
            ))
        })
    }

    pub async fn pages<P, T>(
        &mut self,
        url: &str,
        token_param: &str,
        what: &str,
        split: fn(P) -> (Vec<T>, Option<String>),
    ) -> Result<Vec<T>, AppError>
    where
        P: DeserializeOwned,
    {
        let mut all = Vec::new();
        let mut next: Option<String> = None;
        for _ in 0..self.brand.max_pages {
            let mut page_url = url.to_string();
            if let Some(token) = &next {
                page_url.push_str(&format!("&{token_param}={}", urlencoding::encode(token)));
            }
            let reply = self.send(Method::GET, &page_url, None).await?;
            let (items, token) = split(self.decode::<P>(&reply.text)?);
            all.extend(items);
            match token.filter(|t| !t.is_empty()) {
                Some(token) if next.as_deref() != Some(token.as_str()) => next = Some(token),
                _ => return Ok(all),
            }
        }
        Err(AppError::Internal(format!(
            "{} : plus de {} pages pour {what}",
            self.brand.label, self.brand.max_pages
        )))
    }

    pub async fn send(
        &mut self,
        method: Method,
        url: &str,
        body: Option<String>,
    ) -> Result<Reply, AppError> {
        let mut attempt = 0u32;
        loop {
            if let Some(pacer) = self.pacer.as_mut() {
                pacer.wait_turn().await;
            }
            let mut request = self
                .http
                .request(method.clone(), url)
                .bearer_auth(self.token.as_str())
                .header("Accept", "application/json");
            if let Some(text) = &body {
                request = request
                    .header("Content-Type", "application/json")
                    .body(text.clone());
            }
            let raw = match request.send().await {
                Ok(r) => r,
                Err(e) if attempt + 1 < MAX_ATTEMPTS => {
                    log::warn!("[{}] reseau : {}", self.brand.tag, redact(&e.to_string()));
                    self.pause(BASE_BACKOFF * (1u32 << attempt)).await;
                    attempt += 1;
                    continue;
                }
                Err(e) => return Err(AppError::from(e).redacted()),
            };
            let reply = Reply {
                status: raw.status(),
                headers: raw.headers().clone(),
                text: raw.text().await?,
            };
            match (self.brand.screen)(&reply) {
                Screen::Accept => return Ok(reply),
                Screen::Fail(e) => return Err(e),
                Screen::Continue => {}
            }
            match reply.status {
                s if s.is_success() => return Ok(reply),
                StatusCode::UNAUTHORIZED => self.refresh_after_refusal().await?,
                StatusCode::FORBIDDEN => {
                    return Err(AppError::Forbidden(self.brand.forbidden.to_string()))
                }
                StatusCode::TOO_MANY_REQUESTS => self.throttled(&reply, &mut attempt).await?,
                s if s.is_server_error() && attempt + 1 < MAX_ATTEMPTS => {
                    self.pause(BASE_BACKOFF * (1u32 << attempt)).await;
                    attempt += 1;
                }
                other => {
                    return Err(AppError::Internal(format!(
                        "{} : HTTP {} ({} octets)",
                        self.brand.label,
                        other.as_u16(),
                        reply.text.len()
                    )))
                }
            }
        }
    }

    async fn throttled(&mut self, reply: &Reply, attempt: &mut u32) -> Result<(), AppError> {
        let wait = match (self.brand.throttle)(reply) {
            Throttle::Quota { resume, message } => {
                self.quota_resume = Some(resume);
                return Err(AppError::RateLimit(message));
            }
            Throttle::Wait(wait) => Some(wait),
            Throttle::Default => self.advised_wait(reply),
        };
        if *attempt + 1 >= MAX_ATTEMPTS {
            return Err(AppError::RateLimit(format!(
                "{} apres {MAX_ATTEMPTS} tentatives",
                self.brand.rate_label
            )));
        }
        self.pause(wait.unwrap_or(BASE_BACKOFF * (1u32 << *attempt)))
            .await;
        *attempt += 1;
        Ok(())
    }

    fn advised_wait(&self, reply: &Reply) -> Option<Duration> {
        let raw = self
            .brand
            .wait_headers
            .iter()
            .find_map(|name| reply.header(name))?;
        let seconds: u64 = raw.split(',').next()?.trim().parse().ok()?;
        Some(Duration::from_secs(seconds).min(RETRY_AFTER_CAP))
    }

    async fn refresh_after_refusal(&mut self) -> Result<(), AppError> {
        let label = self.brand.label;
        let Some(r) = &self.refresh else {
            return Err(AppError::Unauthorized(self.brand.unauthorized.to_string()));
        };
        if self.refreshed {
            return Err(AppError::Unauthorized(format!(
                "{label} : jeton refuse apres rafraichissement"
            )));
        }
        self.refreshed = true;
        let failure = r.failure;
        self.token = refresh_locked(r.spec, &r.base, &r.pool, r.vault.as_ref(), &self.token)
            .await
            .map_err(|e| match failure {
                Some(f) => f(e),
                None => AppError::Unauthorized(format!(
                    "jeton refuse, rafraichissement impossible : {e}"
                )),
            })?;
        Ok(())
    }
}
