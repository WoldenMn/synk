use crate::error::AppError;
use serde::Serialize;
use sqlx::SqlitePool;
use tauri::State;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct WithingsDevice {
    pub id: i64,
    pub device_id: String,
    pub device_type: Option<String>,
    pub model: Option<String>,
    pub model_id: Option<i64>,
    pub battery: Option<String>,
    pub last_session_date: Option<String>,
    pub first_session_date: Option<String>,
}

#[tauri::command]
pub async fn get_withings_devices(
    pool: State<'_, SqlitePool>,
) -> Result<Vec<WithingsDevice>, AppError> {
    sqlx::query_as::<_, WithingsDevice>(
        "SELECT id, device_id, device_type, model, model_id, battery, last_session_date, first_session_date \
         FROM withings_devices ORDER BY last_session_date DESC",
    )
    .fetch_all(pool.inner())
    .await
    .map_err(AppError::from)
}
