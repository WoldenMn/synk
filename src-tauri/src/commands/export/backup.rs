use crate::error::AppError;
use crate::services::backup::create_backup_in;
use sqlx::SqlitePool;
use tauri::{Manager, State};

#[tauri::command]
pub async fn create_backup(
    app: tauri::AppHandle,
    pool: State<'_, SqlitePool>,
) -> Result<String, AppError> {
    let app_data_dir = app.path().app_data_dir()?;
    create_backup_in(&app_data_dir, Some(pool.inner())).await
}
