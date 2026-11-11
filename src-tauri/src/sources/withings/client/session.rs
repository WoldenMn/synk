use reqwest::Client;
use serde::de::DeserializeOwned;
use sqlx::SqlitePool;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::time::sleep;
use zeroize::Zeroizing;

use super::super::types::WithingsResponse;
use super::tokens::refresh_locked;
use crate::error::AppError;
use crate::sources::withings::config::{API_BASE_BACKOFF_MS, API_MAX_RETRIES};
use crate::utils::http;
use crate::utils::redact::redact;
use crate::utils::text::truncate_chars;

fn parse_failure(e: &serde_json::Error, text: &str) -> AppError {
    log::debug!(
        "[withings] reponse non decodee : {}",
        truncate_chars(&redact(text), 300)
    );
    AppError::Internal(format!(
        "JSON parse error: {} ({} octets)",
        redact(&e.to_string()),
        text.len()
    ))
}

pub const API_BASE: &str = "https://wbsapi.withings.net";

static API_SEMAPHORE: tokio::sync::OnceCell<Arc<Semaphore>> = tokio::sync::OnceCell::const_new();

async fn get_semaphore() -> Arc<Semaphore> {
    API_SEMAPHORE
        .get_or_init(|| async { Arc::new(Semaphore::new(4)) })
        .await
        .clone()
}

pub(crate) fn envelope_status(text: &str) -> Option<i64> {
    #[derive(serde::Deserialize)]
    struct Envelope {
        status: i64,
    }
    serde_json::from_str::<Envelope>(text)
        .ok()
        .map(|e| e.status)
}

enum Attempt<T> {
    Done(WithingsResponse<T>),
    Retry(AppError),
    Refused,
    Fail(AppError),
}

#[derive(Clone)]
pub struct WithingsClient {
    http: Client,
    base_url: String,
    access_token: Zeroizing<String>,
    pool: Option<SqlitePool>,
    vault: Option<Arc<crate::crypto::CryptoVault>>,
}

impl WithingsClient {
    pub fn at(base: &str, access_token: Zeroizing<String>) -> Result<Self, AppError> {
        Ok(Self {
            http: http::client(base, Duration::from_secs(30))?,
            base_url: base.to_string(),
            access_token,
            pool: None,
            vault: None,
        })
    }

    pub fn with_pool(mut self, pool: SqlitePool) -> Self {
        self.pool = Some(pool);
        self
    }

    pub fn with_vault(mut self, vault: Arc<crate::crypto::CryptoVault>) -> Self {
        self.vault = Some(vault);
        self
    }

    pub async fn post<T: DeserializeOwned>(
        &self,
        path: &str,
        params: &[(&str, &str)],
    ) -> Result<WithingsResponse<T>, AppError> {
        let sem = get_semaphore().await;
        let _permit = sem
            .acquire()
            .await
            .map_err(|e| AppError::Internal(format!("Semaphore acquire failed: {}", e)))?;

        let url = format!("{}{}", self.base_url, path);
        let mut last_err: Option<AppError> = None;
        let mut current_token = self.access_token.clone();
        let mut refreshed = false;
        let mut attempt = 0u32;

        while attempt < API_MAX_RETRIES {
            if attempt > 0 {
                let backoff = API_BASE_BACKOFF_MS * (1u64 << (attempt - 1));
                sleep(Duration::from_millis(backoff)).await;
            }
            match self.send_once::<T>(&url, &current_token, params).await {
                Attempt::Done(parsed) => return Ok(parsed),
                Attempt::Fail(e) => return Err(e),
                Attempt::Retry(e) => {
                    last_err = Some(e);
                    attempt += 1;
                }
                Attempt::Refused => {
                    current_token = self
                        .refresh_after_refusal(&current_token, refreshed)
                        .await?;
                    refreshed = true;
                }
            }
        }

        let summary = |reason: &str| format!("All {} retries failed: {}", API_MAX_RETRIES, reason);
        Err(match last_err {
            Some(AppError::RateLimit(reason)) => AppError::RateLimit(summary(&reason)),
            Some(other) => AppError::Internal(summary(&other.to_string())),
            None => AppError::Internal(summary("")),
        })
    }

    async fn send_once<T: DeserializeOwned>(
        &self,
        url: &str,
        token: &str,
        params: &[(&str, &str)],
    ) -> Attempt<T> {
        let resp = match self
            .http
            .post(url)
            .bearer_auth(token)
            .form(params)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => return Attempt::Retry(AppError::from(e)),
        };
        let status_code = resp.status();
        let text = match resp.text().await {
            Ok(t) => t,
            Err(e) => return Attempt::Retry(AppError::from(e)),
        };

        if status_code == reqwest::StatusCode::UNAUTHORIZED || envelope_status(&text) == Some(401) {
            return Attempt::Refused;
        }
        if status_code == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Attempt::Retry(AppError::RateLimit(format!(
                "HTTP 429 ({} octets)",
                text.len()
            )));
        }
        let parsed: WithingsResponse<T> = match serde_json::from_str(&text) {
            Ok(p) => p,
            Err(e) => return Attempt::Retry(parse_failure(&e, &text)),
        };
        match parsed.status {
            0 => Attempt::Done(parsed),
            601 => Attempt::Retry(AppError::RateLimit(
                "Withings code 601: rate limit".to_string(),
            )),
            503 => Attempt::Fail(AppError::Internal(
                "Withings code 503: parametres de requete invalides".to_string(),
            )),
            other => Attempt::Fail(AppError::Internal(format!(
                "Withings code {other}: API error"
            ))),
        }
    }

    async fn refresh_after_refusal(
        &self,
        current_token: &str,
        already_refreshed: bool,
    ) -> Result<Zeroizing<String>, AppError> {
        let (Some(pool), Some(vault)) = (&self.pool, &self.vault) else {
            return Err(AppError::Unauthorized(
                "Withings code 401: token invalid".to_string(),
            ));
        };
        if already_refreshed {
            return Err(AppError::Unauthorized(
                "Withings code 401: jeton refuse apres rafraichissement".to_string(),
            ));
        }
        refresh_locked(&self.base_url, pool, vault.as_ref(), current_token)
            .await
            .map_err(|e| {
                AppError::Unauthorized(format!("jeton refuse, rafraichissement impossible : {e}"))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::parse_failure;
    use crate::sources::withings::types::WithingsResponse;

    #[test]
    fn a_parse_failure_names_the_size_not_the_body() {
        let text = r#"{"status":0,"body":"refresh_token=RT-FUITE-3"}"#;
        let e = serde_json::from_str::<WithingsResponse<Vec<i64>>>(text).unwrap_err();
        let msg = parse_failure(&e, text).to_string();
        assert!(!msg.contains("FUITE"), "{msg}");
        assert!(msg.contains(&format!("{} octets", text.len())), "{msg}");
    }
}
