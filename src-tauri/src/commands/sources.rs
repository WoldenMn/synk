use crate::crypto::CryptoVault;
use crate::error::AppError;
use crate::services::sources::{self as svc, SourceInfo};
use crate::sources::oauth::start_authorization;
use crate::sources::sync::{sync_sources, Progress, SourceOutcome};
use crate::sources::{all_sources, source_by_name, SourceKind};
use sqlx::SqlitePool;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

#[derive(Clone, serde::Serialize)]
struct SyncProgress {
    source: &'static str,
    etape: u32,
    total: u32,
    libelle: &'static str,
}

#[tauri::command]
pub async fn list_sources(
    pool: State<'_, SqlitePool>,
    vault: State<'_, Arc<CryptoVault>>,
) -> Result<Vec<SourceInfo>, AppError> {
    svc::list(pool.inner(), vault.inner().as_ref()).await
}

#[tauri::command]
pub async fn source_start_auth(
    source: String,
    pool: State<'_, SqlitePool>,
    vault: State<'_, Arc<CryptoVault>>,
    app: AppHandle,
) -> Result<String, AppError> {
    match source_by_name(&source)?.kind() {
        SourceKind::OAuth(spec) => {
            start_authorization(spec, pool.inner(), vault.inner(), app).await
        }
        SourceKind::File(_) => Err(AppError::Validation(format!(
            "la source {source} ne se connecte pas par OAuth"
        ))),
    }
}

#[tauri::command]
pub async fn source_disconnect(
    source: String,
    pool: State<'_, SqlitePool>,
    vault: State<'_, Arc<CryptoVault>>,
) -> Result<svc::DisconnectReport, AppError> {
    svc::disconnect(pool.inner(), vault.inner(), &source).await
}

#[tauri::command]
pub async fn source_set_credentials(
    source: String,
    client_id: String,
    client_secret: String,
    pool: State<'_, SqlitePool>,
    vault: State<'_, Arc<CryptoVault>>,
) -> Result<(), AppError> {
    svc::set_credentials(
        pool.inner(),
        vault.inner().as_ref(),
        &source,
        &client_id,
        &client_secret,
    )
    .await
}

fn progress_emitter(app: &AppHandle) -> impl Fn(Progress) + Sync + '_ {
    move |p: Progress| {
        let payload = SyncProgress {
            source: p.source,
            etape: p.etape,
            total: p.total,
            libelle: p.libelle,
        };
        if let Err(e) = app.emit("sync-progress", payload) {
            log::warn!(
                "[sync] progression {}/{} non emise : {e}",
                p.source,
                p.etape
            );
        }
    }
}

#[tauri::command]
pub async fn sync_all_sources(
    app: AppHandle,
    pool: State<'_, SqlitePool>,
    vault: State<'_, Arc<CryptoVault>>,
) -> Result<Vec<SourceOutcome>, AppError> {
    crate::db::gate::ensure_idle()?;
    Ok(sync_sources(
        all_sources(),
        pool.inner(),
        vault.inner(),
        &progress_emitter(&app),
    )
    .await)
}

#[tauri::command]
pub async fn sync_source(
    source: String,
    app: AppHandle,
    pool: State<'_, SqlitePool>,
    vault: State<'_, Arc<CryptoVault>>,
) -> Result<Vec<SourceOutcome>, AppError> {
    svc::sync_one(
        &source,
        pool.inner(),
        vault.inner(),
        &progress_emitter(&app),
    )
    .await
}

#[tauri::command]
pub fn get_invalidated_secrets(
    invalidated: State<'_, crate::crypto::InvalidatedSecrets>,
) -> crate::crypto::InvalidatedSecrets {
    invalidated.inner().clone()
}
