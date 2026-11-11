use crate::error::AppError;
use crate::services::insights::{self as svc, Insight, InsightReport};
use sqlx::SqlitePool;
use tauri::State;

#[tauri::command]
pub async fn get_insights(
    pool: State<'_, SqlitePool>,
    limit: Option<i64>,
) -> Result<Vec<Insight>, AppError> {
    svc::recent(pool.inner(), limit).await
}

#[tauri::command]
pub async fn mark_insights_seen(pool: State<'_, SqlitePool>) -> Result<(), AppError> {
    svc::mark_seen(pool.inner()).await
}

#[tauri::command]
pub async fn generate_insights(pool: State<'_, SqlitePool>) -> Result<InsightReport, AppError> {
    Ok(svc::run_generation(pool.inner()).await)
}
