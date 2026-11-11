use std::sync::Arc;

use chrono::NaiveDate;
use reqwest::{Method, StatusCode};
use serde::de::DeserializeOwned;
use sqlx::SqlitePool;

use super::spec::OAUTH;
use super::types::{Page, Rollup, Rollups};
use crate::crypto::CryptoVault;
use crate::error::AppError;
use crate::sources::oauth::brand::{plain_throttle, Brand, Refresh, Reply, Screen};
use crate::sources::oauth::rest::RestClient;
use crate::sources::SourceId;

const NOT_LINKED: &str = "Google Health : ce compte Google n'est pas lie a Google Health. \
Ouvrez l'application Google Health avec ce compte (ou migrez votre compte Fitbit vers Google), \
puis reconnectez";
const REFRESH_DEAD: &str = "Google Health : jeton de rafraichissement expire ou revoque \
(7 jours en mode Testing du projet Google Cloud) : reconnectez Google Health";

fn refresh_failure(e: AppError) -> AppError {
    if e.to_string().contains("invalid_grant") {
        AppError::Unauthorized(REFRESH_DEAD.to_string())
    } else {
        AppError::Unauthorized(format!("jeton refuse, rafraichissement impossible : {e}"))
    }
}

fn is_not_linked(body: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|v| v["error"]["details"].as_array().cloned())
        .is_some_and(|details| {
            details
                .iter()
                .any(|d| d["reason"].as_str() == Some("ACCOUNT_NOT_LINKED"))
        })
}

fn screen(reply: &Reply) -> Screen {
    if reply.status == StatusCode::BAD_REQUEST && is_not_linked(&reply.text) {
        return Screen::Fail(AppError::Unauthorized(NOT_LINKED.to_string()));
    }
    Screen::Continue
}

static BRAND: Brand = Brand {
    label: "Google Health",
    tag: "google_health",
    forbidden: "Google Health refuse l'acces : portee non accordee, ou compte Fitbit \
                pas encore migre vers Google",
    unauthorized: "Google Health : jeton refuse",
    rate_label: "Google Health : limite de debit atteinte",
    wait_headers: &["retry-after"],
    max_pages: 200,
    screen,
    throttle: plain_throttle,
};

fn civil(date: NaiveDate) -> serde_json::Value {
    use chrono::Datelike;
    serde_json::json!({
        "date": {"year": date.year(), "month": date.month(), "day": date.day()},
        "time": {}
    })
}

pub struct GoogleClient {
    rest: RestClient,
    api_base: String,
}

impl GoogleClient {
    pub async fn connect(
        token_base: &str,
        api_base: &str,
        pool: &SqlitePool,
        vault: &Arc<CryptoVault>,
    ) -> Result<Self, AppError> {
        let refresh = Refresh {
            spec: &OAUTH,
            base: token_base.to_string(),
            pool: pool.clone(),
            vault: vault.clone(),
            failure: Some(refresh_failure),
        };
        Ok(Self {
            rest: RestClient::connect(
                &BRAND,
                SourceId::GOOGLE_HEALTH,
                api_base,
                pool,
                vault,
                Some(refresh),
            )
            .await?,
            api_base: api_base.to_string(),
        })
    }

    pub async fn verify_identity(&mut self) -> Result<(), AppError> {
        let url = format!("{}/v4/users/me/identity", self.api_base);
        self.rest.send(Method::GET, &url, None).await.map(|_| ())
    }

    pub async fn list<P: DeserializeOwned>(
        &mut self,
        data_type: &str,
        filter: &str,
        page_size: u32,
    ) -> Result<Vec<P>, AppError> {
        let url = format!(
            "{}/v4/users/me/dataTypes/{data_type}/dataPoints?pageSize={page_size}&filter={}",
            self.api_base,
            urlencoding::encode(filter)
        );
        self.rest
            .pages(&url, "pageToken", data_type, |page: Page<P>| {
                (page.data_points, page.next_page_token)
            })
            .await
    }

    pub async fn daily_rollup(
        &mut self,
        data_type: &str,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<Rollup>, AppError> {
        let url = format!(
            "{}/v4/users/me/dataTypes/{data_type}/dataPoints:dailyRollUp",
            self.api_base
        );
        let body = serde_json::json!({
            "range": {"start": civil(from), "end": civil(to)},
            "windowSizeDays": 1
        });
        let reply = self
            .rest
            .send(Method::POST, &url, Some(body.to_string()))
            .await?;
        Ok(self.rest.decode::<Rollups>(&reply.text)?.rollup_data_points)
    }
}
