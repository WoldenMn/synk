use serde::Serialize;
use sqlx::SqlitePool;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

mod auto_backup;
pub mod container;
mod periodic_backup;
mod post_ready;
pub mod runner;
mod schedule;

pub use auto_backup::{backup_before_migration, read_current_schema_version, BACKUP_ROLLING_KEEP};
pub use periodic_backup::*;
pub use post_ready::{spawn_post_ready_auto_purge, spawn_post_ready_seed_then_backup};
pub use schedule::{run_every, BACKUP_RECHECK_INTERVAL};

#[derive(Debug, Serialize, Clone, PartialEq)]
#[serde(tag = "kind", rename_all = "PascalCase")]
pub enum BootError {
    AppDataDir { message: String },
    DbBusyAfterRetry { attempts: u32 },
    DbLocked { message: String },
    MigrationFailed { version: u32, message: String },
    CryptoUnavailable { message: String },
    CryptoMigrationFailed { message: String },
    BackupFailed { message: String },
    DevOnRealData { identifier: String },
}

#[derive(Debug, Serialize, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct IntegrityWarning {
    pub details: String,
}

pub fn resolve_app_data_dir(handle: &AppHandle) -> Result<PathBuf, BootError> {
    let dir = handle
        .path()
        .app_data_dir()
        .map_err(|e| BootError::AppDataDir {
            message: format!("Tauri path resolution failed: {e}"),
        })?;
    ensure_dir_exists(&dir)
}

pub fn ensure_dir_exists(dir: &Path) -> Result<PathBuf, BootError> {
    std::fs::create_dir_all(dir).map_err(|e| BootError::AppDataDir {
        message: format!("Cannot create app data dir {}: {}", dir.display(), e),
    })?;
    Ok(dir.to_path_buf())
}

pub const EXPECTED_LATEST_VERSION: i64 = crate::db::LATEST_VERSION;
pub const PROD_IDENTIFIER: &str = "com.synk.health";

pub fn ensure_not_dev_on_real_data(identifier: &str, debug_build: bool) -> Result<(), BootError> {
    if debug_build && identifier == PROD_IDENTIFIER {
        return Err(BootError::DevOnRealData {
            identifier: identifier.to_string(),
        });
    }
    Ok(())
}
pub const MAX_RETRIES: u32 = 3;
pub const BASE_BACKOFF_MS: u64 = 300;

fn is_busy_error(e: &sqlx::Error) -> bool {
    let msg = e.to_string().to_lowercase();
    msg.contains("database is locked") || msg.contains("sqlite_busy")
}

// MAX_RETRIES tentatives au total (initiale + (MAX_RETRIES - 1) backoffs).
// Avec MAX_RETRIES=3 et BASE_BACKOFF_MS=300 -> sleeps de 300ms puis 600ms (900ms cumules avant exit).
// Suffit pour absorber un double-lancement transitoire ; au-dela on remonte DbBusyAfterRetry.
pub async fn init_pool_with_retry(db_path: PathBuf) -> Result<SqlitePool, BootError> {
    for attempt in 0..MAX_RETRIES {
        if attempt > 0 {
            let backoff = BASE_BACKOFF_MS * (1u64 << (attempt - 1));
            tokio::time::sleep(std::time::Duration::from_millis(backoff)).await;
        }
        match crate::db::init_pool(db_path.clone()).await {
            Ok(pool) => return Ok(pool),
            Err(crate::db::InitError::Migration(e)) if is_busy_error(&e.source) => {
                log::warn!(
                    "init_pool busy (attempt {}/{}): {e}",
                    attempt + 1,
                    MAX_RETRIES
                );
                continue;
            }
            Err(crate::db::InitError::Migration(e)) => {
                return Err(BootError::MigrationFailed {
                    version: e.version as u32,
                    message: e.source.to_string(),
                });
            }
            Err(crate::db::InitError::Sqlx(e)) if is_busy_error(&e) => {
                log::warn!(
                    "init_pool busy (attempt {}/{}): {e}",
                    attempt + 1,
                    MAX_RETRIES
                );
                continue;
            }
            Err(crate::db::InitError::Sqlx(e)) => {
                return Err(BootError::DbLocked {
                    message: e.to_string(),
                });
            }
        }
    }
    Err(BootError::DbBusyAfterRetry {
        attempts: MAX_RETRIES,
    })
}

pub async fn integrity_check(pool: &SqlitePool) -> Option<IntegrityWarning> {
    let result: Result<Vec<(String,)>, _> = sqlx::query_as("PRAGMA integrity_check")
        .fetch_all(pool)
        .await;
    match result {
        Ok(rows) if rows.len() == 1 && rows[0].0 == "ok" => None,
        Ok(rows) => Some(IntegrityWarning {
            details: rows
                .into_iter()
                .take(20)
                .map(|r| r.0)
                .collect::<Vec<_>>()
                .join(" ; "),
        }),
        Err(e) => {
            log::warn!("integrity_check failed to execute: {e}");
            Some(IntegrityWarning {
                details: format!("contrôle impossible : {e}"),
            })
        }
    }
}

pub struct BootOutput {
    pub pool: SqlitePool,
    pub integrity_warning: Option<IntegrityWarning>,
    pub vault: std::sync::Arc<crate::crypto::CryptoVault>,
    pub invalidated_secrets: crate::crypto::InvalidatedSecrets,
}

pub async fn run(handle: AppHandle) -> Result<BootOutput, BootError> {
    let identifier = handle.config().identifier.clone();
    log::info!("[boot::run] step 0/7 : identifiant {identifier}");
    ensure_not_dev_on_real_data(&identifier, cfg!(debug_assertions))?;

    log::info!("[boot::run] step 1/7 : resolve_app_data_dir");
    let app_data_dir = resolve_app_data_dir(&handle)?;
    log::info!("[boot::run] app_data_dir = {}", app_data_dir.display());
    let db_path = app_data_dir.join("synk.db");
    log::info!(
        "[boot::run] db_path = {} (exists={})",
        db_path.display(),
        db_path.exists()
    );

    log::info!("[boot::run] step 2/7 : backup_before_migration");
    // Une migration ne part jamais sans sa sauvegarde : si la copie echoue,
    // le boot s'arrete et l'ecran d'erreur propose la restauration.
    if let Some(path) = backup_before_migration(&db_path, &app_data_dir).await? {
        log::info!("[boot::run] auto-backup written to {}", path.display());
    }

    log::info!("[boot::run] step 3/7 : init_pool_with_retry");
    let pool = init_pool_with_retry(db_path).await?;
    log::info!("[boot::run] pool initialized");

    log::info!("[boot::run] step 4/7 : integrity_check");
    let warning = integrity_check(&pool).await;
    log::info!(
        "[boot::run] integrity={}",
        if warning.is_some() { "WARNING" } else { "OK" }
    );

    log::info!("[boot::run] step 5/7 : crypto_init");
    let vault =
        match crate::crypto::CryptoVault::init(crate::crypto::keyring::service_for(&identifier)) {
            Ok(v) => std::sync::Arc::new(v),
            Err(e) => {
                pool.close().await;
                return Err(BootError::CryptoUnavailable {
                    message: e.to_string(),
                });
            }
        };
    log::info!("[boot::run] crypto vault ready");

    log::info!("[boot::run] step 6/7 : invalidate_undecryptable_secrets");
    let invalidated_secrets =
        match crate::crypto::invalidate_undecryptable_secrets(&pool, &vault).await {
            Ok(r) => r,
            Err(e) => {
                pool.close().await;
                return Err(BootError::CryptoMigrationFailed {
                    message: format!("invalidation des secrets indechiffrables : {e}"),
                });
            }
        };
    log::info!(
        "[boot::run] undecryptable secrets: {} source(s) touchee(s) {:?}",
        invalidated_secrets.sources.len(),
        invalidated_secrets
            .sources
            .iter()
            .map(|s| s.source.as_str())
            .collect::<Vec<_>>()
    );

    log::info!("[boot::run] step 7/7 : migrate_plaintext_secrets");
    let report = match crate::crypto::migrate_plaintext_secrets(&pool, &vault).await {
        Ok(r) => r,
        Err(e) => {
            pool.close().await;
            return Err(BootError::CryptoMigrationFailed {
                message: e.to_string(),
            });
        }
    };
    if report.auth_rows_encrypted > 0 || report.credentials_rows_encrypted > 0 {
        log::info!(
            "[boot::run] crypto migration: auth={}, credentials={}",
            report.auth_rows_encrypted,
            report.credentials_rows_encrypted
        );
    } else {
        log::info!("[boot::run] crypto migration: nothing to migrate");
    }

    Ok(BootOutput {
        pool,
        integrity_warning: warning,
        vault,
        invalidated_secrets,
    })
}
