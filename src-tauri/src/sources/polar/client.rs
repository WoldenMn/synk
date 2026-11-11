use std::sync::Arc;

use reqwest::{Method, StatusCode};
use serde::de::DeserializeOwned;
use sqlx::SqlitePool;

use crate::crypto::CryptoVault;
use crate::error::AppError;
use crate::sources::oauth::brand::{plain_throttle, Brand, Reply, Screen};
use crate::sources::oauth::rest::RestClient;
use crate::sources::SourceId;

fn screen(reply: &Reply) -> Screen {
    if reply.status == StatusCode::CONFLICT {
        return Screen::Accept;
    }
    Screen::Continue
}

static BRAND: Brand = Brand {
    label: "Polar",
    tag: "polar",
    forbidden:
        "Polar refuse l'acces : consentements obligatoires non acceptes sur account.polar.com",
    unauthorized: "Polar : jeton refuse ou revoque, reconnecte Polar",
    rate_label: "Polar : limite de debit atteinte",
    wait_headers: &["retry-after", "ratelimit-reset"],
    max_pages: 1,
    screen,
    throttle: plain_throttle,
};

pub struct PolarClient {
    rest: RestClient,
    base: String,
}

pub enum Registration {
    Created,
    AlreadyRegistered,
}

impl PolarClient {
    pub async fn connect(
        base: &str,
        pool: &SqlitePool,
        vault: &Arc<CryptoVault>,
    ) -> Result<Self, AppError> {
        Ok(Self {
            rest: RestClient::connect(&BRAND, SourceId::POLAR, base, pool, vault, None).await?,
            base: base.to_string(),
        })
    }

    pub async fn register(&mut self) -> Result<Registration, AppError> {
        if self.rest.user_ref().is_empty() {
            return Err(AppError::Unauthorized(
                "Polar : identifiant d'utilisateur absent, reconnecte Polar".to_string(),
            ));
        }
        let body = serde_json::json!({ "member-id": format!("synk-{}", self.rest.user_ref()) });
        let url = format!("{}/v3/users", self.base);
        let reply = self
            .rest
            .send(Method::POST, &url, Some(body.to_string()))
            .await?;
        match reply.status {
            StatusCode::CONFLICT => Ok(Registration::AlreadyRegistered),
            StatusCode::NO_CONTENT => Err(AppError::Validation(
                "Polar : utilisateur introuvable a l'enregistrement".to_string(),
            )),
            _ => Ok(Registration::Created),
        }
    }

    pub async fn get<T: DeserializeOwned>(&mut self, path: &str) -> Result<Option<T>, AppError> {
        let url = format!("{}{path}", self.base);
        let reply = self.rest.send(Method::GET, &url, None).await?;
        if reply.status == StatusCode::NO_CONTENT {
            return Ok(None);
        }
        self.rest.decode(&reply.text).map(Some)
    }
}
