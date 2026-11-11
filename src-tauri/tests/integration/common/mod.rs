#![allow(dead_code)]

pub mod capture;

use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use std::sync::Arc;
use synk_lib::crypto::CryptoVault;

pub async fn get_pair(
    pool: &SqlitePool,
    vault: &CryptoVault,
) -> Result<(zeroize::Zeroizing<String>, zeroize::Zeroizing<String>), synk_lib::error::AppError> {
    synk_lib::sources::store::get_credentials(pool, vault, synk_lib::sources::SourceId::WITHINGS)
        .await
}

pub async fn set_pair(
    pool: &SqlitePool,
    vault: &CryptoVault,
    client_id: &str,
    client_secret: &str,
) -> Result<(), synk_lib::error::AppError> {
    synk_lib::sources::store::set_credentials(
        pool,
        vault,
        synk_lib::sources::SourceId::WITHINGS,
        client_id,
        client_secret,
    )
    .await
}

pub async fn memory_pool() -> SqlitePool {
    let pool = SqlitePool::connect(":memory:").await.unwrap();
    synk_lib::db::bootstrap_schema(&pool).await.unwrap();
    pool
}

pub fn test_vault() -> Arc<CryptoVault> {
    Arc::new(CryptoVault::from_key([0x7a; 32]))
}

pub async fn memory_pool_with_foreign_keys() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .after_connect(|conn, _| {
            Box::pin(async move {
                sqlx::query("PRAGMA foreign_keys=ON")
                    .execute(&mut *conn)
                    .await?;
                Ok(())
            })
        })
        .connect(":memory:")
        .await
        .unwrap();
    synk_lib::db::bootstrap_schema(&pool).await.unwrap();
    pool
}

pub async fn memory_pool_with_source(id: &str) -> SqlitePool {
    let pool = memory_pool().await;
    sqlx::query("INSERT INTO sources (id, label, kind) VALUES (?1, ?1, 'oauth')")
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();
    pool
}
