use std::sync::Arc;

use serde::de::DeserializeOwned;
use sqlx::SqlitePool;

use super::spec::OAUTH;
use super::types::Page;
use crate::crypto::CryptoVault;
use crate::error::AppError;
use crate::sources::oauth::brand::{keep_going, plain_throttle};
use crate::sources::oauth::brand::{Brand, Refresh};
use crate::sources::oauth::rest::RestClient;
use crate::sources::SourceId;

static BRAND: Brand = Brand {
    label: "Oura",
    tag: "oura",
    forbidden: "Oura refuse l'acces : abonnement Oura expire ou portee non accordee",
    unauthorized: "Oura : jeton refuse",
    rate_label: "Oura : limite de debit atteinte",
    wait_headers: &["retry-after"],
    max_pages: 200,
    screen: keep_going,
    throttle: plain_throttle,
};

pub struct OuraClient {
    rest: RestClient,
    base: String,
}

impl OuraClient {
    pub async fn connect(
        base: &str,
        pool: &SqlitePool,
        vault: &Arc<CryptoVault>,
    ) -> Result<Self, AppError> {
        let refresh = Refresh {
            spec: &OAUTH,
            base: base.to_string(),
            pool: pool.clone(),
            vault: vault.clone(),
            failure: None,
        };
        Ok(Self {
            rest: RestClient::connect(&BRAND, SourceId::OURA, base, pool, vault, Some(refresh))
                .await?,
            base: base.to_string(),
        })
    }

    pub async fn collection<T: DeserializeOwned>(
        &mut self,
        route: &str,
        start: &str,
        end: &str,
    ) -> Result<Vec<T>, AppError> {
        let url = format!(
            "{}/v2/usercollection/{route}?start_date={}&end_date={}",
            self.base,
            urlencoding::encode(start),
            urlencoding::encode(end)
        );
        self.rest
            .pages(&url, "next_token", route, |page: Page<T>| {
                (page.data, page.next_token)
            })
            .await
    }
}
