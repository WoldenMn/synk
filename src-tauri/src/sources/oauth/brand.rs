use std::sync::Arc;
use std::time::Duration;

use reqwest::StatusCode;
use sqlx::SqlitePool;

use super::spec::OAuthSpec;
use crate::crypto::CryptoVault;
use crate::error::AppError;

pub struct Reply {
    pub status: StatusCode,
    pub text: String,
    pub(super) headers: reqwest::header::HeaderMap,
}

impl std::fmt::Debug for Reply {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Reply({}, {} octets)", self.status, self.text.len())
    }
}

impl Reply {
    pub fn header(&self, name: &str) -> Option<String> {
        self.headers.get(name)?.to_str().ok().map(str::to_string)
    }
}

pub enum Screen {
    Continue,
    Accept,
    Fail(AppError),
}

pub enum Throttle {
    Default,
    Wait(Duration),
    Quota { resume: Duration, message: String },
}

pub struct Brand {
    pub label: &'static str,
    pub tag: &'static str,
    pub forbidden: &'static str,
    pub unauthorized: &'static str,
    pub rate_label: &'static str,
    pub wait_headers: &'static [&'static str],
    pub max_pages: usize,
    pub screen: fn(&Reply) -> Screen,
    pub throttle: fn(&Reply) -> Throttle,
}

pub fn keep_going(_: &Reply) -> Screen {
    Screen::Continue
}

pub fn plain_throttle(_: &Reply) -> Throttle {
    Throttle::Default
}

pub struct Refresh {
    pub spec: &'static OAuthSpec,
    pub base: String,
    pub pool: SqlitePool,
    pub vault: Arc<CryptoVault>,
    pub failure: Option<fn(AppError) -> AppError>,
}
