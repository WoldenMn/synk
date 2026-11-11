pub mod google_health;
pub mod import;
mod model;
pub mod oauth;
pub mod oura;
pub mod polar;
pub mod store;
pub mod sync;
pub mod whoop;
pub mod withings;

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use sqlx::SqlitePool;

use crate::crypto::CryptoVault;
use crate::error::AppError;

pub use model::{Domain, FileSpec, MetricDefinition, Observation, SourceId, SourceKind, SyncBatch};

pub type RevokeFuture<'a> = Pin<Box<dyn Future<Output = Result<bool, AppError>> + Send + 'a>>;

pub struct RevokeContext<'a> {
    pub pool: &'a SqlitePool,
    pub vault: &'a Arc<CryptoVault>,
}

pub type SyncFuture<'a> = Pin<Box<dyn Future<Output = Result<SyncBatch, AppError>> + Send + 'a>>;

pub type ImportFuture<'a> =
    Pin<Box<dyn Future<Output = Result<import::pipeline::ImportReport, AppError>> + Send + 'a>>;

pub struct ImportContext<'a> {
    pub pool: &'a SqlitePool,
    pub path: &'a str,
    pub cancel: Arc<std::sync::atomic::AtomicBool>,
    pub on_progress: &'a (dyn Fn(import::pipeline::ImportProgress) + Sync),
}

pub struct SyncContext<'a> {
    pub pool: &'a SqlitePool,
    pub vault: &'a Arc<CryptoVault>,
    pub on_progress: &'a (dyn Fn(u32, &'static str) + Sync),
}

pub trait Source: Send + Sync {
    fn id(&self) -> SourceId;
    fn label(&self) -> &'static str;
    fn kind(&self) -> SourceKind;
    fn capabilities(&self) -> &'static [Domain];
    fn steps(&self) -> u32 {
        1
    }
    fn developer_url(&self) -> Option<&'static str> {
        None
    }
    fn metric_definitions(&self) -> &'static [MetricDefinition] {
        &[]
    }
    fn sync<'a>(&'a self, _ctx: &'a SyncContext<'a>) -> SyncFuture<'a> {
        let message = format!(
            "{} s'importe depuis un fichier, il n'a pas de synchronisation",
            self.label()
        );
        Box::pin(async move { Err(AppError::Validation(message)) })
    }
    fn import<'a>(&'a self, _ctx: ImportContext<'a>) -> Option<ImportFuture<'a>> {
        None
    }
    fn revoke<'a>(&'a self, _ctx: &'a RevokeContext<'a>) -> RevokeFuture<'a> {
        Box::pin(async { Ok(false) })
    }
}

static SOURCES: &[&dyn Source] = &[
    &withings::WITHINGS,
    &oura::OURA,
    &polar::POLAR,
    &whoop::WHOOP,
    &google_health::GOOGLE_HEALTH,
    &import::APPLE_HEALTH,
    &import::fit::FIT,
    &import::trace::TRACE,
    &import::table::CSV,
];

pub fn all_sources() -> &'static [&'static dyn Source] {
    SOURCES
}

pub fn find_source(id: &str) -> Option<&'static dyn Source> {
    all_sources()
        .iter()
        .copied()
        .find(|s| s.id().as_str() == id)
}

pub fn label_of(id: SourceId) -> &'static str {
    find_source(id.as_str()).map_or(id.as_str(), |s| s.label())
}

pub fn source_by_name(id: &str) -> Result<&'static dyn Source, AppError> {
    find_source(id).ok_or_else(|| AppError::Validation(format!("source inconnue : {id}")))
}

pub fn require_source(id: SourceId) -> Result<&'static dyn Source, AppError> {
    all_sources()
        .iter()
        .copied()
        .find(|s| s.id() == id)
        .ok_or_else(|| AppError::Internal(format!("source {} absente du registre", id.as_str())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_registry_lists_every_source_with_its_domains() {
        let ids: Vec<&str> = all_sources().iter().map(|s| s.id().as_str()).collect();
        assert_eq!(
            ids,
            vec![
                "withings",
                "oura",
                "polar",
                "whoop",
                "google_health",
                "apple_health",
                "fit",
                "trace",
                "csv"
            ]
        );
        let withings = require_source(SourceId::WITHINGS).expect("withings");
        assert_eq!(withings.label(), "Withings");
        assert!(
            matches!(withings.kind(), SourceKind::OAuth(spec) if spec.id == SourceId::WITHINGS)
        );
        assert_eq!(withings.capabilities().len(), 8);
        assert_eq!(withings.steps(), 6);
        assert!(!withings.capabilities().contains(&Domain::Glucose));
        let oura = require_source(SourceId::OURA).expect("oura");
        assert_eq!(oura.label(), "Oura");
        assert_eq!(oura.steps(), 4);
        assert_eq!(oura.metric_definitions().len(), 3);
        assert!(matches!(oura.kind(), SourceKind::OAuth(spec) if spec.pkce));
        let polar = require_source(SourceId::POLAR).expect("polar");
        assert_eq!(polar.label(), "Polar");
        assert_eq!(polar.steps(), 3);
        assert_eq!(polar.metric_definitions().len(), 2);
        assert!(matches!(polar.kind(), SourceKind::OAuth(spec) if !spec.pkce));
        let whoop = require_source(SourceId::WHOOP).expect("whoop");
        assert_eq!(whoop.label(), "Whoop");
        assert_eq!(whoop.steps(), 3);
        assert_eq!(whoop.metric_definitions().len(), 3);
        assert!(
            matches!(whoop.kind(), SourceKind::OAuth(spec) if !spec.pkce && spec.scopes.contains("offline"))
        );
        let google = require_source(SourceId::GOOGLE_HEALTH).expect("google_health");
        assert_eq!(google.label(), "Google Health");
        assert_eq!(google.steps(), 4);
        assert!(google.metric_definitions().is_empty());
        let apple = require_source(SourceId::APPLE_HEALTH).expect("apple_health");
        assert_eq!(apple.label(), "Apple Santé");
        assert_eq!(apple.metric_definitions().len(), 2);
        assert!(matches!(
            apple.kind(),
            SourceKind::File(spec) if spec.extensions == ["zip", "xml"]
        ));
        let fit = require_source(SourceId::FIT).expect("fit");
        assert_eq!(fit.label(), "Fichiers FIT");
        assert!(matches!(
            fit.kind(),
            SourceKind::File(spec) if spec.extensions == ["fit"]
        ));
        let csv = require_source(SourceId::CSV).expect("csv");
        assert_eq!(csv.label(), "Fichiers CSV");
        assert!(matches!(
            csv.kind(),
            SourceKind::File(spec) if spec.extensions == ["csv", "txt"]
        ));
        let trace = require_source(SourceId::TRACE).expect("trace");
        assert_eq!(trace.label(), "Traces GPX et TCX");
        assert!(matches!(
            trace.kind(),
            SourceKind::File(spec) if spec.extensions == ["gpx", "tcx"]
        ));
        assert!(matches!(
            google.kind(),
            SourceKind::OAuth(spec) if spec.pkce && spec.refresh_lifetime_secs == Some(604_800)
        ));
    }

    #[test]
    fn every_source_has_a_unique_id_and_a_label() {
        let mut ids: Vec<&str> = all_sources().iter().map(|s| s.id().as_str()).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), all_sources().len());
        assert!(all_sources().iter().all(|s| !s.label().is_empty()));
    }

    #[test]
    fn a_source_is_found_by_its_id_string_and_labelled() {
        assert!(find_source("withings").is_some());
        assert!(find_source("inconnue").is_none());
        assert_eq!(label_of(SourceId::WITHINGS), "Withings");
        assert_eq!(label_of(SourceId::new("inconnue")), "inconnue");
    }

    #[test]
    fn an_unregistered_source_is_an_error_not_a_panic() {
        let err = require_source(SourceId::new("inconnue"))
            .err()
            .expect("erreur");
        assert!(err.to_string().contains("inconnue"), "{err}");
    }
}
