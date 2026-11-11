use crate::error::AppError;
use crate::services::sync_log::{self as svc, SyncLogEntry};
use sqlx::SqlitePool;
use tauri::State;

#[tauri::command]
pub async fn get_sync_log(
    pool: State<'_, SqlitePool>,
    limit: Option<i64>,
) -> Result<Vec<SyncLogEntry>, AppError> {
    svc::recent(pool.inner(), limit).await
}

#[tauri::command]
pub async fn get_last_successful_sync(
    pool: State<'_, SqlitePool>,
) -> Result<Option<String>, AppError> {
    svc::last_successful_sync(pool.inner()).await
}

#[tauri::command]
pub async fn clear_sync_log(pool: State<'_, SqlitePool>) -> Result<(), AppError> {
    svc::clear_sync_log(pool.inner()).await
}
