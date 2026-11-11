use std::sync::Arc;
use std::time::Instant;

use serde::Serialize;
use sqlx::SqlitePool;

use super::{store, Source, SourceId, SourceKind, SyncContext};
use crate::crypto::CryptoVault;
use crate::error::AppError;
use crate::services::sync_log::{record_sync, SyncLogRecord};

#[derive(Debug, Clone, Copy)]
pub struct Progress {
    pub source: &'static str,
    pub etape: u32,
    pub total: u32,
    pub libelle: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct SourceOutcome {
    pub source: String,
    pub label: String,
    pub count: usize,
    pub errors: Vec<String>,
    pub failed: bool,
}

pub async fn logged_step<T, C, F, Fut>(
    pool: &SqlitePool,
    source: SourceId,
    data_type: &str,
    count_of: C,
    f: F,
) -> Result<T, AppError>
where
    C: Fn(&T) -> i64,
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, AppError>>,
{
    let started = chrono::Utc::now().to_rfc3339();
    let clock = Instant::now();
    let res = f().await.map_err(AppError::redacted);
    let took = i64::try_from(clock.elapsed().as_millis()).unwrap_or(i64::MAX);
    let (records_added, status, error) = match &res {
        Ok(v) => (count_of(v), "success", None),
        Err(e) => (0, "error", Some(e.to_string())),
    };
    record_sync(
        pool,
        &SyncLogRecord {
            source: source.as_str(),
            data_type,
            started_at: &started,
            duration_ms: took,
            records_added,
            status,
            error: error.as_deref(),
        },
    )
    .await;
    res
}

async fn is_due(source: &dyn Source, pool: &SqlitePool) -> Result<bool, AppError> {
    match source.kind() {
        SourceKind::OAuth(_) => store::is_connected(pool, source.id()).await,
        SourceKind::File(_) => Ok(false),
    }
}

fn failure(source: &dyn Source, error: AppError) -> SourceOutcome {
    let message = error.redacted().to_string();
    SourceOutcome {
        source: source.id().as_str().to_string(),
        label: source.label().to_string(),
        count: 0,
        errors: vec![format!("{} : {message}", source.label())],
        failed: true,
    }
}

async fn log_failure(
    pool: &SqlitePool,
    source: &dyn Source,
    started: &str,
    took_ms: i64,
    outcome: &SourceOutcome,
) {
    record_sync(
        pool,
        &SyncLogRecord {
            source: source.id().as_str(),
            data_type: "sync",
            started_at: started,
            duration_ms: took_ms,
            records_added: 0,
            status: "error",
            error: outcome.errors.first().map(String::as_str),
        },
    )
    .await;
}

async fn run_one(
    source: &dyn Source,
    pool: &SqlitePool,
    vault: &Arc<CryptoVault>,
    on_progress: &(dyn Fn(Progress) + Sync),
) -> SourceOutcome {
    let id = source.id().as_str();
    let total = source.steps();
    let report = |etape: u32, libelle: &'static str| {
        on_progress(Progress {
            source: id,
            etape,
            total,
            libelle,
        })
    };
    let ctx = SyncContext {
        pool,
        vault,
        on_progress: &report,
    };
    let started = chrono::Utc::now().to_rfc3339();
    let clock = Instant::now();
    match source.sync(&ctx).await {
        Ok(batch) => SourceOutcome {
            source: id.to_string(),
            label: source.label().to_string(),
            count: batch.written.iter().map(|(_, n)| n).sum(),
            errors: batch
                .errors
                .iter()
                .map(|e| format!("{} : {e}", source.label()))
                .collect(),
            failed: false,
        },
        Err(e) => {
            let outcome = failure(source, e);
            let took = i64::try_from(clock.elapsed().as_millis()).unwrap_or(i64::MAX);
            log_failure(pool, source, &started, took, &outcome).await;
            outcome
        }
    }
}

pub const ANALYSES: &str = "analyses";

async fn run_analyses(
    on_progress: &(dyn Fn(Progress) + Sync),
    pool: &SqlitePool,
) -> Option<SourceOutcome> {
    on_progress(Progress {
        source: ANALYSES,
        etape: 1,
        total: 1,
        libelle: "Calcul",
    });
    let report = crate::services::insights::run_generation(pool).await;
    if report.failed.is_empty() {
        return None;
    }
    Some(SourceOutcome {
        source: ANALYSES.to_string(),
        label: "Analyses".to_string(),
        count: 0,
        errors: report
            .failed
            .iter()
            .map(|name| format!("Analyses : {name}"))
            .collect(),
        failed: false,
    })
}

pub async fn sync_sources(
    sources: &[&dyn Source],
    pool: &SqlitePool,
    vault: &Arc<CryptoVault>,
    on_progress: &(dyn Fn(Progress) + Sync),
) -> Vec<SourceOutcome> {
    let mut outcomes = Vec::new();
    for source in sources {
        match is_due(*source, pool).await {
            Ok(false) => {}
            Ok(true) => outcomes.push(run_one(*source, pool, vault, on_progress).await),
            Err(e) => outcomes.push(failure(*source, e)),
        }
    }
    if outcomes.iter().any(|o| !o.failed) {
        outcomes.extend(run_analyses(on_progress, pool).await);
    }
    outcomes
}
