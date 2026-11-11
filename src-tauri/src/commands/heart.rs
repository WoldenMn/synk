use crate::error::AppError;
use crate::utils::dates::day_range;
use serde::Serialize;
use sqlx::SqlitePool;
use tauri::State;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct HeartRecord {
    pub id: i64,
    pub timestamp: String,
    pub source: String,
    pub ecg_status: Option<i64>,
    pub heart_rate: Option<i64>,
    pub qrs_duration: Option<i64>,
}

#[tauri::command]
pub async fn get_heart_records(
    pool: State<'_, SqlitePool>,
    from: String,
    to: String,
) -> Result<Vec<HeartRecord>, AppError> {
    heart_records_between(pool.inner(), &from, &to).await
}

pub async fn heart_records_between(
    pool: &SqlitePool,
    from: &str,
    to: &str,
) -> Result<Vec<HeartRecord>, AppError> {
    let range = day_range(from, to)?;
    let (from_d, to_d) = (range.from, range.before);
    sqlx::query_as::<_, HeartRecord>(
        "SELECT id, timestamp, source, ecg_status, heart_rate, qrs_duration \
         FROM heart_records WHERE timestamp >= ?1 AND timestamp < ?2 \
         ORDER BY timestamp DESC",
    )
    .bind(&from_d)
    .bind(&to_d)
    .fetch_all(pool)
    .await
    .map_err(AppError::from)
}

pub async fn latest_heart_record(pool: &SqlitePool) -> Result<Option<HeartRecord>, AppError> {
    let row = sqlx::query_as::<_, HeartRecord>(
        "SELECT id, timestamp, source, ecg_status, heart_rate, qrs_duration FROM heart_records ORDER BY timestamp DESC LIMIT 1",
    )
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

#[tauri::command]
pub async fn get_latest_heart_record(
    pool: State<'_, SqlitePool>,
) -> Result<Option<HeartRecord>, AppError> {
    latest_heart_record(pool.inner()).await
}
