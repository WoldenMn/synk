use sqlx::SqlitePool;
use std::sync::Arc;
use zeroize::Zeroizing;

use super::session::{WithingsClient, API_BASE};
use crate::error::AppError;
use crate::sources::withings::spec::OAUTH;
use crate::sources::withings::types::TokenBody;
use crate::sources::{oauth, store, SourceId};

pub async fn save_tokens(
    pool: &SqlitePool,
    tokens: &TokenBody,
    vault: &crate::crypto::CryptoVault,
) -> Result<(), AppError> {
    store::save_auth(pool, vault, SourceId::WITHINGS, &tokens.to_set()).await
}

pub async fn refresh_locked(
    base: &str,
    pool: &SqlitePool,
    vault: &crate::crypto::CryptoVault,
    stale_access: &str,
) -> Result<Zeroizing<String>, AppError> {
    oauth::refresh_locked(&OAUTH, base, pool, vault, stale_access).await
}

pub async fn get_valid_client(
    pool: &SqlitePool,
    vault: &Arc<crate::crypto::CryptoVault>,
) -> Result<WithingsClient, AppError> {
    get_valid_client_at(API_BASE, pool, vault).await
}

pub async fn get_valid_client_at(
    base: &str,
    pool: &SqlitePool,
    vault: &Arc<crate::crypto::CryptoVault>,
) -> Result<WithingsClient, AppError> {
    let stored = store::load_auth(pool, vault.as_ref(), SourceId::WITHINGS)
        .await?
        .ok_or_else(|| AppError::Unauthorized("No Withings authentication found".to_string()))?;
    let mut access = stored.access_token;
    if stored.expires_at < chrono::Utc::now().timestamp() + 300 {
        access = refresh_locked(base, pool, vault.as_ref(), &access).await?;
    }
    Ok(WithingsClient::at(base, access)?
        .with_pool(pool.clone())
        .with_vault(vault.clone()))
}
