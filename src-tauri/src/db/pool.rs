use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::PathBuf;

use super::bootstrap::bootstrap_schema;
use super::migrations::InitError;

pub async fn init_pool(db_path: PathBuf) -> Result<SqlitePool, InitError> {
    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                sqlx::query("PRAGMA foreign_keys=ON")
                    .execute(&mut *conn)
                    .await?;
                sqlx::query("PRAGMA synchronous=NORMAL")
                    .execute(&mut *conn)
                    .await?;
                sqlx::query(sqlx::AssertSqlSafe(format!(
                    "PRAGMA busy_timeout={}",
                    super::gate::BUSY_TIMEOUT_MS
                )))
                .execute(&mut *conn)
                .await?;
                Ok(())
            })
        })
        .before_acquire(|conn, _meta| {
            Box::pin(async move {
                sqlx::query(sqlx::AssertSqlSafe(format!(
                    "PRAGMA busy_timeout={}",
                    super::gate::busy_timeout_ms()
                )))
                .execute(&mut *conn)
                .await?;
                Ok(true)
            })
        })
        .connect(&db_url)
        .await?;

    sqlx::query("PRAGMA journal_mode=WAL")
        .execute(&pool)
        .await?;

    bootstrap_schema(&pool).await?;
    if let Err(e) = super::maintenance::optimize_on_open(&pool).await {
        log::warn!("[db] PRAGMA optimize a l'ouverture en echec : {e}");
    }

    Ok(pool)
}
