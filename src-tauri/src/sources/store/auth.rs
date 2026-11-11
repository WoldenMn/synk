use sqlx::SqlitePool;
use zeroize::Zeroizing;

use crate::crypto::CryptoVault;
use crate::error::AppError;
use crate::sources::oauth::TokenSet;
use crate::sources::{label_of, SourceId};

pub struct StoredAuth {
    pub access_token: Zeroizing<String>,
    pub refresh_token: Zeroizing<String>,
    pub expires_at: i64,
    pub user_ref: String,
}

fn unreadable(field: &str, source: SourceId, cause: AppError) -> AppError {
    AppError::Crypto(format!(
        "Vault decrypt {field} failed: {cause}. Reconnecte ton compte {}.",
        label_of(source)
    ))
}

fn open_auth(
    vault: &CryptoVault,
    source: SourceId,
    row: (String, String, i64, String),
) -> Result<StoredAuth, AppError> {
    let (access, refresh, expires_at, user_ref) = row;
    Ok(StoredAuth {
        access_token: vault
            .open_str(&access)
            .map_err(|e| unreadable("access_token", source, e))?,
        refresh_token: vault
            .open_str(&refresh)
            .map_err(|e| unreadable("refresh_token", source, e))?,
        expires_at,
        user_ref,
    })
}

pub async fn load_auth(
    pool: &SqlitePool,
    vault: &CryptoVault,
    source: SourceId,
) -> Result<Option<StoredAuth>, AppError> {
    let row: Option<(String, String, i64, String)> = sqlx::query_as(
        "SELECT access_token, refresh_token, expires_at, COALESCE(user_ref, '') FROM source_auth WHERE source_id = ?1",
    )
    .bind(source.as_str())
    .fetch_optional(pool)
    .await?;
    row.map(|row| open_auth(vault, source, row)).transpose()
}

pub async fn save_auth(
    pool: &SqlitePool,
    vault: &CryptoVault,
    source: SourceId,
    tokens: &TokenSet,
) -> Result<(), AppError> {
    let expires_at = chrono::Utc::now().timestamp() + tokens.expires_in;
    let access = vault.seal_str(&tokens.access_token)?;
    let refresh = vault.seal_str(&tokens.refresh_token)?;

    sqlx::query(
        "INSERT INTO source_auth \
         (source_id, access_token, refresh_token, expires_at, token_type, user_ref, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now')) \
         ON CONFLICT(source_id) DO UPDATE SET \
         access_token = excluded.access_token, \
         refresh_token = excluded.refresh_token, \
         expires_at = excluded.expires_at, \
         token_type = excluded.token_type, \
         user_ref = excluded.user_ref, \
         updated_at = datetime('now')",
    )
    .bind(source.as_str())
    .bind(&access)
    .bind(&refresh)
    .bind(expires_at)
    .bind(&tokens.token_type)
    .bind(&tokens.user_ref)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_auth(pool: &SqlitePool, source: SourceId) -> Result<(), AppError> {
    sqlx::query("DELETE FROM source_auth WHERE source_id = ?1")
        .bind(source.as_str())
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn is_connected(pool: &SqlitePool, source: SourceId) -> Result<bool, AppError> {
    let found: Option<(i64,)> =
        sqlx::query_as("SELECT 1 FROM source_auth WHERE source_id = ?1 AND refresh_token != ''")
            .bind(source.as_str())
            .fetch_optional(pool)
            .await?;
    Ok(found.is_some())
}
