use crate::error::AppError;
use crate::validation::check_limit;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

const MAX_ERROR_CHARS: usize = 2_000;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SyncLogEntry {
    pub id: i64,
    pub source: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub data_type: String,
    pub duration_ms: Option<i64>,
    pub records_added: Option<i64>,
    pub status: String,
    pub error: Option<String>,
}

pub async fn recent(pool: &SqlitePool, limit: Option<i64>) -> Result<Vec<SyncLogEntry>, AppError> {
    let limit = check_limit(limit, 100)?;
    let rows = sqlx::query_as::<_, SyncLogEntry>(
        "SELECT id, source, started_at, finished_at, data_type, duration_ms, records_added, status, error FROM sync_log ORDER BY started_at DESC LIMIT ?1",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn last_successful_sync(pool: &SqlitePool) -> Result<Option<String>, AppError> {
    let row: (Option<String>,) = sqlx::query_as(
        "SELECT strftime('%Y-%m-%dT%H:%M:%SZ', MIN(last_success)) FROM (\
            SELECT MAX(finished_at) AS last_success FROM sync_log \
            WHERE status = 'success' GROUP BY data_type)",
    )
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

pub async fn last_success_of(pool: &SqlitePool, source: &str) -> Result<Option<String>, AppError> {
    let row: (Option<String>,) = sqlx::query_as(
        "SELECT strftime('%Y-%m-%dT%H:%M:%SZ', MAX(finished_at)) FROM sync_log \
         WHERE source = ?1 AND status = 'success'",
    )
    .bind(source)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

pub async fn clear_sync_log(pool: &SqlitePool) -> Result<(), AppError> {
    sqlx::query(
        "DELETE FROM sync_log WHERE id NOT IN (\
            SELECT id FROM (\
                SELECT id, ROW_NUMBER() OVER (\
                    PARTITION BY source, data_type ORDER BY finished_at DESC, id DESC) AS rank \
                FROM sync_log WHERE status = 'success') \
            WHERE rank = 1)",
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn purge(pool: &SqlitePool, keep_rows: i64, keep_days: i64) -> Result<u64, AppError> {
    let result = sqlx::query(
        "DELETE FROM sync_log \
         WHERE id NOT IN (\
            SELECT id FROM (\
                SELECT id, ROW_NUMBER() OVER (\
                    PARTITION BY source, data_type ORDER BY finished_at DESC, id DESC) AS rank \
                FROM sync_log WHERE status = 'success') \
            WHERE rank = 1) \
         AND (started_at < date('now', ?1) \
            OR id NOT IN (SELECT id FROM sync_log ORDER BY started_at DESC, id DESC LIMIT ?2))",
    )
    .bind(format!("-{keep_days} days"))
    .bind(keep_rows)
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

pub struct SyncLogRecord<'a> {
    pub source: &'a str,
    pub data_type: &'a str,
    pub started_at: &'a str,
    pub duration_ms: i64,
    pub records_added: i64,
    pub status: &'a str,
    pub error: Option<&'a str>,
}

pub async fn record_sync(pool: &SqlitePool, record: &SyncLogRecord<'_>) {
    if let Err(e) = sqlx::query(
        "INSERT INTO sync_log (source, started_at, finished_at, data_type, duration_ms, records_added, status, error) \
         VALUES (?1, ?2, datetime('now'), ?3, ?4, ?5, ?6, ?7)",
    )
    .bind(record.source)
    .bind(record.started_at)
    .bind(record.data_type)
    .bind(record.duration_ms)
    .bind(record.records_added)
    .bind(record.status)
    .bind(
        record
            .error
            .map(|e| crate::utils::text::truncate_chars(e, MAX_ERROR_CHARS)),
    )
    .execute(pool)
    .await
    {
        log::warn!(
            "[sync_log] entree {}/{} non enregistree : {e}",
            record.source,
            record.data_type
        );
    }
}
