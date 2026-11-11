use crate::error::AppError;
use crate::metric_repository::MetricPoint;
use serde::Serialize;
use sqlx::SqlitePool;
use tauri::State;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct MetricDefinition {
    pub id: i64,
    pub slug: String,
    pub label: String,
    pub unit: String,
    pub category: String,
    pub withings_type: Option<i64>,
    pub is_builtin: i64,
    pub sort_order: i64,
    pub decimals: i64,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
}

#[tauri::command]
pub async fn get_metric_definitions(
    pool: State<'_, SqlitePool>,
    category: Option<String>,
) -> Result<Vec<MetricDefinition>, AppError> {
    let query = match category {
        Some(cat) => sqlx::query_as::<_, MetricDefinition>(
            "SELECT id, slug, label, unit, category, withings_type, is_builtin, sort_order, decimals, min_value, max_value
             FROM metric_definitions WHERE category = ? ORDER BY sort_order"
        ).bind(cat),
        None => sqlx::query_as::<_, MetricDefinition>(
            "SELECT id, slug, label, unit, category, withings_type, is_builtin, sort_order, decimals, min_value, max_value
             FROM metric_definitions ORDER BY sort_order"
        ),
    };
    query.fetch_all(pool.inner()).await.map_err(AppError::from)
}

#[tauri::command]
pub async fn get_metric_series(
    pool: State<'_, SqlitePool>,
    slug: String,
    from: String,
    to: String,
) -> Result<Vec<MetricPoint>, AppError> {
    crate::services::metrics::select_series(pool.inner(), &slug, &from, &to).await
}

#[tauri::command]
pub async fn get_metric_latest(
    pool: State<'_, SqlitePool>,
    slug: String,
) -> Result<Option<MetricPoint>, AppError> {
    crate::services::metrics::select_latest(pool.inner(), &slug).await
}

#[tauri::command]
pub async fn get_metric_priorities(
    pool: State<'_, SqlitePool>,
) -> Result<Vec<crate::services::metrics::MetricPriority>, AppError> {
    crate::services::metrics::priorities(pool.inner()).await
}

#[tauri::command]
pub async fn set_metric_priority(
    pool: State<'_, SqlitePool>,
    slug: String,
    sources: Vec<String>,
) -> Result<(), AppError> {
    let ordered: Vec<&str> = sources.iter().map(String::as_str).collect();
    crate::services::metrics::set_priority(pool.inner(), &slug, &ordered).await
}

#[tauri::command]
pub async fn get_metric_secondary_points(
    pool: State<'_, SqlitePool>,
    slug: String,
    from: String,
    to: String,
) -> Result<Vec<MetricPoint>, AppError> {
    crate::services::metrics::select_secondary(pool.inner(), &slug, &from, &to).await
}
