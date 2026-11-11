use crate::error::AppError;
use crate::validation::{check_f64_opt, check_one_of};
use sqlx::SqlitePool;
use tauri::State;

pub fn validate_measurement_goal(
    target_value: Option<f64>,
    target_direction: Option<&str>,
) -> Result<(), AppError> {
    check_f64_opt("Cible", target_value, 0.0, 1_000_000.0)?;
    if let Some(direction) = target_direction {
        check_one_of("Sens", direction, &crate::commands::goals::GOAL_DIRECTIONS)?;
    }
    Ok(())
}

#[tauri::command]
pub async fn set_measurement_goal(
    pool: State<'_, SqlitePool>,
    type_id: i64,
    target_value: Option<f64>,
    target_direction: Option<String>,
) -> Result<(), AppError> {
    validate_measurement_goal(target_value, target_direction.as_deref())?;
    sqlx::query(
        "UPDATE measurement_types_legacy SET target_value = ?1, target_direction = ?2 WHERE id = ?3",
    )
    .bind(target_value)
    .bind(&target_direction)
    .bind(type_id)
    .execute(pool.inner())
    .await?;
    Ok(())
}
