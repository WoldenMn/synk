use crate::error::AppError;
use crate::services::error_log::{self as svc, ErrorLogEntry};
use sqlx::SqlitePool;
use tauri::State;

#[tauri::command]
pub async fn log_error(
    pool: State<'_, SqlitePool>,
    source: String,
    code: String,
    message: String,
    context: Option<String>,
) -> Result<(), AppError> {
    svc::log_error(pool.inner(), &source, &code, &message, context.as_deref()).await
}

#[tauri::command]
pub async fn get_error_log(
    pool: State<'_, SqlitePool>,
    limit: i64,
    code_filter: Option<String>,
) -> Result<Vec<ErrorLogEntry>, AppError> {
    svc::get_error_log(pool.inner(), limit, code_filter.as_deref()).await
}

#[tauri::command]
pub async fn purge_error_log(pool: State<'_, SqlitePool>) -> Result<(), AppError> {
    svc::purge_error_log(pool.inner()).await
}
