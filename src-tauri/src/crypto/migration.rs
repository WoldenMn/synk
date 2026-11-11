use sqlx::SqlitePool;

use super::CryptoVault;
use crate::error::AppError;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct MigrationReport {
    pub auth_rows_encrypted: u32,
    pub credentials_rows_encrypted: u32,
}

fn sealed_if_plain(vault: &CryptoVault, value: String) -> Result<Option<String>, AppError> {
    if CryptoVault::is_encrypted(&value) {
        Ok(None)
    } else {
        vault.seal_str(&value).map(Some)
    }
}

pub async fn migrate_plaintext_secrets(
    pool: &SqlitePool,
    vault: &CryptoVault,
) -> Result<MigrationReport, AppError> {
    let mut tx = pool.begin().await.map_err(AppError::from)?;
    let mut report = MigrationReport::default();

    let auth_rows: Vec<(String, String, String)> =
        sqlx::query_as("SELECT source_id, access_token, refresh_token FROM source_auth")
            .fetch_all(&mut *tx)
            .await
            .map_err(AppError::from)?;
    for (source, access, refresh) in auth_rows {
        let new_access = sealed_if_plain(vault, access.clone())?;
        let new_refresh = sealed_if_plain(vault, refresh.clone())?;
        if new_access.is_none() && new_refresh.is_none() {
            continue;
        }
        sqlx::query(
            "UPDATE source_auth SET access_token = ?1, refresh_token = ?2 WHERE source_id = ?3",
        )
        .bind(new_access.unwrap_or(access))
        .bind(new_refresh.unwrap_or(refresh))
        .bind(&source)
        .execute(&mut *tx)
        .await
        .map_err(AppError::from)?;
        report.auth_rows_encrypted += 1;
    }

    let credential_rows: Vec<(String, String, String)> =
        sqlx::query_as("SELECT source_id, client_id, client_secret FROM source_credentials")
            .fetch_all(&mut *tx)
            .await
            .map_err(AppError::from)?;
    for (source, client_id, client_secret) in credential_rows {
        let new_id = sealed_if_plain(vault, client_id.clone())?;
        let new_secret = sealed_if_plain(vault, client_secret.clone())?;
        if new_id.is_none() && new_secret.is_none() {
            continue;
        }
        sqlx::query(
            "UPDATE source_credentials SET client_id = ?1, client_secret = ?2 WHERE source_id = ?3",
        )
        .bind(new_id.unwrap_or(client_id))
        .bind(new_secret.unwrap_or(client_secret))
        .bind(&source)
        .execute(&mut *tx)
        .await
        .map_err(AppError::from)?;
        report.credentials_rows_encrypted += 1;
    }

    tx.commit().await.map_err(AppError::from)?;
    Ok(report)
}
