use crate::error::AppError;
use crate::services::backup_files::{list_backups_inner, BackupInfo};

#[tauri::command]
pub async fn list_backups(app: tauri::AppHandle) -> Result<Vec<BackupInfo>, AppError> {
    use tauri::Manager;
    let dir = app.path().app_data_dir()?;
    list_backups_inner(&dir)
}
