use sqlx::SqlitePool;
use zeroize::Zeroizing;

use crate::crypto::CryptoVault;
use crate::error::AppError;
use crate::sources::{label_of, SourceId};

type Pair = (Zeroizing<String>, Zeroizing<String>);

fn open_pair(vault: &CryptoVault, row: Option<(String, String)>) -> Result<Pair, AppError> {
    let (id_raw, secret_raw) = row.unwrap_or_default();
    let open = |raw: &str| -> Result<Zeroizing<String>, AppError> {
        if raw.is_empty() {
            Ok(Zeroizing::new(String::new()))
        } else {
            vault.open_str(raw)
        }
    };
    Ok((open(&id_raw)?, open(&secret_raw)?))
}

pub async fn get_credentials(
    pool: &SqlitePool,
    vault: &CryptoVault,
    source: SourceId,
) -> Result<Pair, AppError> {
    let row: Option<(String, String)> = sqlx::query_as(
        "SELECT client_id, client_secret FROM source_credentials WHERE source_id = ?1",
    )
    .bind(source.as_str())
    .fetch_optional(pool)
    .await?;
    open_pair(vault, row)
}

pub async fn set_credentials(
    pool: &SqlitePool,
    vault: &CryptoVault,
    source: SourceId,
    client_id: &str,
    client_secret: &str,
) -> Result<(), AppError> {
    let id_enc = vault.seal_str(client_id)?;
    let secret_enc = vault.seal_str(client_secret)?;
    sqlx::query(
        "INSERT INTO source_credentials (source_id, client_id, client_secret, updated_at) \
         VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP) \
         ON CONFLICT(source_id) DO UPDATE SET \
         client_id = excluded.client_id, \
         client_secret = excluded.client_secret, \
         updated_at = CURRENT_TIMESTAMP",
    )
    .bind(source.as_str())
    .bind(&id_enc)
    .bind(&secret_enc)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn require_credentials(
    pool: &SqlitePool,
    vault: &CryptoVault,
    source: SourceId,
) -> Result<Pair, AppError> {
    let (id, secret) = get_credentials(pool, vault, source).await?;
    let label = label_of(source);
    if id.is_empty() {
        return Err(missing_credential(label, "Client ID"));
    }
    if secret.is_empty() {
        return Err(missing_credential(label, "Client Secret"));
    }
    Ok((id, secret))
}

pub async fn require_client_id(
    pool: &SqlitePool,
    vault: &CryptoVault,
    source: SourceId,
) -> Result<Zeroizing<String>, AppError> {
    let (id, _) = get_credentials(pool, vault, source).await?;
    if id.is_empty() {
        return Err(missing_credential(label_of(source), "Client ID"));
    }
    Ok(id)
}

pub async fn require_client_secret(
    pool: &SqlitePool,
    vault: &CryptoVault,
    source: SourceId,
) -> Result<Zeroizing<String>, AppError> {
    let (_, secret) = get_credentials(pool, vault, source).await?;
    if secret.is_empty() {
        return Err(missing_credential(label_of(source), "Client Secret"));
    }
    Ok(secret)
}

fn missing_credential(label: &str, field: &str) -> AppError {
    AppError::Validation(format!(
        "Identifiants {label} manquants : saisissez le {field} dans Paramètres > {label}."
    ))
}
