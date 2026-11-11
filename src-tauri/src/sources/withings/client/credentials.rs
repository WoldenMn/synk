use crate::error::AppError;
use crate::sources::{store, SourceId};
use sqlx::SqlitePool;
use zeroize::Zeroizing;

pub async fn client_id(
    pool: &SqlitePool,
    vault: &crate::crypto::CryptoVault,
) -> Result<Zeroizing<String>, AppError> {
    store::require_client_id(pool, vault, SourceId::WITHINGS).await
}

pub async fn client_secret(
    pool: &SqlitePool,
    vault: &crate::crypto::CryptoVault,
) -> Result<Zeroizing<String>, AppError> {
    store::require_client_secret(pool, vault, SourceId::WITHINGS).await
}
