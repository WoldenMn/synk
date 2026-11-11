use sqlx::SqlitePool;

use crate::error::AppError;
use crate::sources::SourceId;

pub async fn get_last_sync(
    pool: &SqlitePool,
    source: SourceId,
    domain: &str,
) -> Result<i64, AppError> {
    let row: Option<(i64,)> = sqlx::query_as(
        "SELECT last_sync_at FROM source_state WHERE source_id = ?1 AND domain = ?2",
    )
    .bind(source.as_str())
    .bind(domain)
    .fetch_optional(pool)
    .await?;
    Ok(row.map_or(0, |r| r.0))
}

pub async fn set_last_sync(
    pool: &SqlitePool,
    source: SourceId,
    domain: &str,
    cursor: i64,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO source_state (source_id, domain, last_sync_at, updated_at) \
         VALUES (?1, ?2, ?3, datetime('now')) \
         ON CONFLICT(source_id, domain) DO UPDATE SET \
         last_sync_at = excluded.last_sync_at, updated_at = datetime('now')",
    )
    .bind(source.as_str())
    .bind(domain)
    .bind(cursor)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn reset_cursors(pool: &SqlitePool, source: SourceId) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE source_state SET last_sync_at = 0, updated_at = datetime('now') WHERE source_id = ?1",
    )
    .bind(source.as_str())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn seed_cursors(
    pool: &SqlitePool,
    source: SourceId,
    domains: &[&str],
) -> Result<(), AppError> {
    for domain in domains {
        sqlx::query(
            "INSERT OR IGNORE INTO source_state (source_id, domain, last_sync_at) VALUES (?1, ?2, 0)",
        )
        .bind(source.as_str())
        .bind(domain)
        .execute(pool)
        .await?;
    }
    Ok(())
}
