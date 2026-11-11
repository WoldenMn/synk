use crate::error::AppError;
use sqlx::SqlitePool;
use tauri::State;

use super::engine::ImportReport;

pub const MAX_IMPORT_BYTES: u64 = 64 * 1024 * 1024;

fn read_import_file(path: &str) -> Result<serde_json::Map<String, serde_json::Value>, AppError> {
    let canonical = std::fs::canonicalize(path)
        .map_err(|e| AppError::Validation(format!("Chemin invalide : {e}")))?;
    if canonical.extension().map(|e| e != "json").unwrap_or(true) {
        return Err(AppError::Validation(
            "Seuls les fichiers .json sont acceptes".into(),
        ));
    }
    let size = std::fs::metadata(&canonical)?.len();
    if size > MAX_IMPORT_BYTES {
        return Err(AppError::Validation(format!(
            "Fichier trop volumineux : {} Mo, plafond {} Mo",
            size / (1024 * 1024),
            MAX_IMPORT_BYTES / (1024 * 1024)
        )));
    }
    check_first_bytes(&canonical)?;
    let content = std::fs::read_to_string(&canonical)?;
    let data: serde_json::Value = serde_json::from_str(&content)?;
    data.as_object()
        .cloned()
        .ok_or_else(|| AppError::Validation("Format JSON invalide".to_string()))
}

const HEADER_BYTES: usize = 4096;

fn check_first_bytes(path: &std::path::Path) -> Result<(), AppError> {
    use std::io::Read;
    let mut head = [0u8; HEADER_BYTES];
    let read = std::fs::File::open(path)?.read(&mut head)?;
    let first = head[..read].iter().find(|b| !b.is_ascii_whitespace());
    if first == Some(&b'{') {
        Ok(())
    } else {
        Err(AppError::Validation("Format JSON invalide".to_string()))
    }
}

async fn load_import_file(
    path: &str,
) -> Result<serde_json::Map<String, serde_json::Value>, AppError> {
    let path = path.to_string();
    tokio::task::spawn_blocking(move || read_import_file(&path))
        .await
        .map_err(|e| AppError::Internal(format!("lecture de l'import interrompue : {e}")))?
}

pub async fn import_json_inner(pool: &SqlitePool, path: &str) -> Result<ImportReport, AppError> {
    let data = load_import_file(path).await?;
    super::engine::import_value(pool, &data).await
}

#[derive(Debug, serde::Serialize, PartialEq, Eq)]
pub struct ImportPreview {
    pub key: String,
    pub rows: u64,
}

pub fn inspect_import_inner(path: &str) -> Result<Vec<ImportPreview>, AppError> {
    let data = read_import_file(path)?;
    Ok(super::tables::EXPORTED
        .iter()
        .filter_map(|s| {
            data.get(s.key)
                .and_then(|v| v.as_array())
                .map(|rows| ImportPreview {
                    key: s.key.to_string(),
                    rows: rows.len() as u64,
                })
        })
        .collect())
}

#[tauri::command]
pub async fn import_json(
    pool: State<'_, SqlitePool>,
    path: String,
) -> Result<ImportReport, AppError> {
    import_json_inner(pool.inner(), &path).await
}

#[tauri::command]
pub async fn inspect_import_file(path: String) -> Result<Vec<ImportPreview>, AppError> {
    tokio::task::spawn_blocking(move || inspect_import_inner(&path))
        .await
        .map_err(|e| AppError::Internal(format!("lecture de l'import interrompue : {e}")))?
}
