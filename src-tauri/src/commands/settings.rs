use crate::error::AppError;
use tauri::Manager;

#[tauri::command]
pub async fn get_app_data_dir(app: tauri::AppHandle) -> Result<String, AppError> {
    app.path()
        .app_data_dir()
        .map(|p| p.to_string_lossy().into_owned())
        .map_err(|e| AppError::Internal(format!("Dossier de donnees introuvable : {e}")))
}
