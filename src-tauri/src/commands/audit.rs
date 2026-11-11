use crate::error::AppError;
use crate::services::audit::{self as svc, AuditMetrics, AuditPurgeConfig};
use sqlx::SqlitePool;
use tauri::State;

#[tauri::command]
pub async fn get_audit_metrics(pool: State<'_, SqlitePool>) -> Result<AuditMetrics, AppError> {
    svc::get_audit_metrics(pool.inner()).await
}

#[tauri::command]
pub async fn purge_audit_log(
    pool: State<'_, SqlitePool>,
    older_than_days: i64,
) -> Result<i64, AppError> {
    svc::purge_audit_log(pool.inner(), older_than_days).await
}

#[tauri::command]
pub async fn get_audit_purge_config(
    pool: State<'_, SqlitePool>,
) -> Result<AuditPurgeConfig, AppError> {
    svc::get_audit_purge_config(pool.inner()).await
}

#[tauri::command]
pub async fn set_audit_purge_config(
    pool: State<'_, SqlitePool>,
    config: AuditPurgeConfig,
) -> Result<(), AppError> {
    svc::set_audit_purge_config(pool.inner(), config).await
}
