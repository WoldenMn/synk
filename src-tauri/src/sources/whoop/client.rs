use std::sync::Arc;
use std::time::Duration;

use serde::de::DeserializeOwned;
use sqlx::SqlitePool;

use super::spec::OAUTH;
use super::throttle::{classify, Throttle as Window};
use super::types::Page;
use crate::crypto::CryptoVault;
use crate::error::AppError;
use crate::sources::oauth::brand::{keep_going, Brand, Refresh, Reply, Throttle};
use crate::sources::oauth::clock::Clock;
use crate::sources::oauth::rest::RestClient;
use crate::sources::SourceId;

const PAGE_SIZE: u32 = 25;
pub const DAILY_LIMIT: u32 = 10_000;

fn classify_reply(reply: &Reply) -> Throttle {
    let limit = reply.header("x-ratelimit-limit");
    let reset = reply.header("x-ratelimit-reset");
    match classify(limit.as_deref(), reset.as_deref()) {
        Window::Day(resume) => Throttle::Quota {
            resume,
            message: format!("quota du jour atteint ({DAILY_LIMIT} requetes par jour)"),
        },
        Window::Minute(wait) => Throttle::Wait(wait),
    }
}

static BRAND: Brand = Brand {
    label: "Whoop",
    tag: "whoop",
    forbidden: "Whoop refuse l'acces : portee non accordee",
    unauthorized: "Whoop : jeton refuse",
    rate_label: "limite a la minute encore atteinte",
    wait_headers: &[],
    max_pages: 400,
    screen: keep_going,
    throttle: classify_reply,
};

pub struct WhoopClient {
    rest: RestClient,
    base: String,
}

impl WhoopClient {
    pub async fn connect(
        base: &str,
        pool: &SqlitePool,
        vault: &Arc<CryptoVault>,
        clock: Arc<dyn Clock>,
        per_minute: usize,
    ) -> Result<Self, AppError> {
        let refresh = Refresh {
            spec: &OAUTH,
            base: base.to_string(),
            pool: pool.clone(),
            vault: vault.clone(),
            failure: None,
        };
        let rest = RestClient::connect(&BRAND, SourceId::WHOOP, base, pool, vault, Some(refresh))
            .await?
            .paced(clock, per_minute);
        Ok(Self {
            rest,
            base: base.to_string(),
        })
    }

    pub fn quota_resume(&self) -> Option<Duration> {
        self.rest.quota_resume()
    }

    pub async fn collection<T: DeserializeOwned>(
        &mut self,
        route: &str,
        start: &str,
    ) -> Result<Vec<T>, AppError> {
        let url = format!(
            "{}/developer/v2/{route}?limit={PAGE_SIZE}&start={}",
            self.base,
            urlencoding::encode(start)
        );
        self.rest
            .pages(&url, "nextToken", route, |page: Page<T>| {
                (page.records, page.next_token)
            })
            .await
    }
}
