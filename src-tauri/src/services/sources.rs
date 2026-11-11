use std::sync::Arc;

use serde::Serialize;
use sqlx::SqlitePool;

use crate::crypto::CryptoVault;
use crate::error::AppError;
use crate::services::sync_log::last_success_of;
use crate::sources::oauth::reauth::{reauth_status, REFRESH_EXPIRY};
use crate::sources::sync::{sync_sources, Progress, SourceOutcome};
use crate::sources::{
    all_sources, source_by_name, store, RevokeContext, Source, SourceId, SourceKind,
};

#[derive(Debug, Serialize)]
pub struct SourceInfo {
    pub id: &'static str,
    pub label: &'static str,
    pub kind: &'static str,
    pub domains: Vec<&'static str>,
    pub connected: bool,
    pub credentials_configured: bool,
    pub last_sync_at: Option<String>,
    pub redirect_uri: Option<String>,
    pub developer_url: Option<&'static str>,
    pub reauth: Option<ReauthInfo>,
    pub extensions: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
pub struct ReauthInfo {
    pub status: &'static str,
    pub expires_at: i64,
}

async fn describe(
    source: &dyn Source,
    pool: &SqlitePool,
    vault: &CryptoVault,
    now: i64,
) -> Result<SourceInfo, AppError> {
    let id = source.id();
    let (kind, connected, credentials_configured, redirect_uri) = match source.kind() {
        SourceKind::OAuth(spec) => {
            let (client_id, client_secret) = store::get_credentials(pool, vault, id).await?;
            (
                "oauth",
                store::is_connected(pool, id).await?,
                !client_id.is_empty() && !client_secret.is_empty(),
                Some(spec.redirect_uri()),
            )
        }
        SourceKind::File(_) => ("file", false, false, None),
    };
    Ok(SourceInfo {
        id: id.as_str(),
        label: source.label(),
        kind,
        domains: source.capabilities().iter().map(|d| d.as_str()).collect(),
        connected,
        credentials_configured,
        last_sync_at: last_success_of(pool, id.as_str()).await?,
        redirect_uri,
        developer_url: source.developer_url(),
        reauth: reauth_of(pool, id, now).await?,
        extensions: match source.kind() {
            SourceKind::File(spec) => spec.extensions.to_vec(),
            SourceKind::OAuth(_) => Vec::new(),
        },
    })
}

async fn reauth_of(
    pool: &SqlitePool,
    id: SourceId,
    now: i64,
) -> Result<Option<ReauthInfo>, AppError> {
    let expires_at = store::get_last_sync(pool, id, REFRESH_EXPIRY).await?;
    Ok(reauth_status(expires_at, now).map(|status| ReauthInfo {
        status: status.as_str(),
        expires_at,
    }))
}

pub async fn list(pool: &SqlitePool, vault: &CryptoVault) -> Result<Vec<SourceInfo>, AppError> {
    list_at(pool, vault, chrono::Utc::now().timestamp()).await
}

pub async fn list_at(
    pool: &SqlitePool,
    vault: &CryptoVault,
    now: i64,
) -> Result<Vec<SourceInfo>, AppError> {
    let mut out = Vec::new();
    for source in all_sources() {
        out.push(describe(*source, pool, vault, now).await?);
    }
    Ok(out)
}

#[derive(Debug, Serialize)]
pub struct DisconnectReport {
    pub revoked: bool,
    pub warning: Option<String>,
}

pub async fn disconnect(
    pool: &SqlitePool,
    vault: &Arc<CryptoVault>,
    source: &str,
) -> Result<DisconnectReport, AppError> {
    disconnect_with(source_by_name(source)?, pool, vault).await
}

pub async fn disconnect_with(
    source: &dyn Source,
    pool: &SqlitePool,
    vault: &Arc<CryptoVault>,
) -> Result<DisconnectReport, AppError> {
    let id = source.id();
    let (revoked, warning) = if store::is_connected(pool, id).await? {
        match source.revoke(&RevokeContext { pool, vault }).await {
            Ok(done) => (done, None),
            Err(e) => {
                log::warn!("[{}] revocation de l'acces en echec : {e}", id.as_str());
                (false, Some(e.redacted().detail()))
            }
        }
    } else {
        (false, None)
    };
    store::delete_auth(pool, id).await?;
    store::reset_cursors(pool, id).await?;
    Ok(DisconnectReport { revoked, warning })
}

const MAX_CREDENTIAL_CHARS: usize = 512;

fn clean_credential<'a>(what: &str, value: &'a str) -> Result<&'a str, AppError> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > MAX_CREDENTIAL_CHARS {
        return Err(AppError::Validation(format!(
            "{what} : entre 1 et {MAX_CREDENTIAL_CHARS} caractères"
        )));
    }
    if value.chars().any(char::is_control) {
        return Err(AppError::Validation(format!(
            "{what} : caractère de contrôle refusé"
        )));
    }
    Ok(value)
}

pub async fn sync_one(
    source: &str,
    pool: &SqlitePool,
    vault: &Arc<CryptoVault>,
    on_progress: &(dyn Fn(Progress) + Sync),
) -> Result<Vec<SourceOutcome>, AppError> {
    let found = source_by_name(source)?;
    if matches!(found.kind(), SourceKind::File(_)) {
        return Err(AppError::Validation(format!(
            "la source {source} est un fichier : elle s'importe, elle ne se synchronise pas"
        )));
    }
    crate::db::gate::ensure_idle()?;
    Ok(sync_sources(&[found], pool, vault, on_progress).await)
}

pub async fn set_credentials(
    pool: &SqlitePool,
    vault: &CryptoVault,
    source: &str,
    client_id: &str,
    client_secret: &str,
) -> Result<(), AppError> {
    let id = source_by_name(source)?.id();
    let client_id = clean_credential("identifiant client", client_id)?;
    let client_secret = clean_credential("secret client", client_secret)?;
    store::set_credentials(pool, vault, id, client_id, client_secret).await
}
