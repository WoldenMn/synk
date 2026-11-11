use crate::error::AppError;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::State;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Goal {
    pub id: i64,
    pub metric_key: String,
    pub target_value: f64,
    pub direction: String,
}

#[derive(Debug, Deserialize)]
pub struct UpsertGoalInput {
    pub metric_key: String,
    pub target_value: f64,
    pub direction: String,
}

#[tauri::command]
pub async fn get_goals(pool: State<'_, SqlitePool>) -> Result<Vec<Goal>, AppError> {
    sqlx::query_as::<_, Goal>(
        "SELECT id, metric_key, target_value, direction FROM goals ORDER BY metric_key ASC",
    )
    .fetch_all(pool.inner())
    .await
    .map_err(AppError::from)
}

pub const GOAL_DIRECTIONS: [&str; 3] = ["decrease", "increase", "maintain"];

pub async fn upsert_goal_inner(pool: &SqlitePool, data: &UpsertGoalInput) -> Result<(), AppError> {
    if !GOAL_DIRECTIONS.contains(&data.direction.as_str()) {
        return Err(AppError::Validation(format!(
            "sens d'objectif inconnu : {}",
            data.direction
        )));
    }
    if !data.target_value.is_finite() || data.target_value <= 0.0 {
        return Err(AppError::Validation(format!(
            "valeur cible invalide : {}",
            data.target_value
        )));
    }
    sqlx::query(
        "INSERT INTO goals (metric_key, target_value, direction, updated_at) \
         VALUES (?1, ?2, ?3, datetime('now')) \
         ON CONFLICT(metric_key) DO UPDATE SET \
         target_value = excluded.target_value, \
         direction = excluded.direction, \
         updated_at = datetime('now')",
    )
    .bind(&data.metric_key)
    .bind(data.target_value)
    .bind(&data.direction)
    .execute(pool)
    .await?;
    Ok(())
}

#[tauri::command]
pub async fn upsert_goal(
    pool: State<'_, SqlitePool>,
    data: UpsertGoalInput,
) -> Result<(), AppError> {
    upsert_goal_inner(pool.inner(), &data).await
}

#[tauri::command]
pub async fn delete_goal(pool: State<'_, SqlitePool>, metric_key: String) -> Result<(), AppError> {
    sqlx::query("DELETE FROM goals WHERE metric_key = ?1")
        .bind(&metric_key)
        .execute(pool.inner())
        .await?;
    crate::services::audit::record_audit_or_warn(
        pool.inner(),
        "delete",
        "goals",
        None,
        Some(&metric_key),
    )
    .await;
    Ok(())
}
