use serde::Serialize;

use sqlx::SqlitePool;
use tauri::{AppHandle, Emitter, State};

use crate::error::AppError;
pub use crate::services::import::{
    import_csv_file as import_csv_inner, import_file as import_file_inner, ImportControl,
};
use crate::sources::import::pipeline::{ImportProgress, ImportReport};
use crate::sources::import::table::store::{self, SavedMapping};
use crate::sources::import::table::targets::{self, TargetInfo};
use crate::sources::import::table::{inspect_file, CsvConfig, Inspection};

#[derive(Serialize)]
pub struct CsvInspection {
    #[serde(flatten)]
    pub inspection: Inspection,
    pub saved: Vec<SavedMapping>,
    pub unreadable_mappings: usize,
    pub targets: Vec<TargetInfo>,
}

pub async fn inspect_csv_inner(pool: &SqlitePool, path: &str) -> Result<CsvInspection, AppError> {
    let owned = path.to_string();
    let inspection = tauri::async_runtime::spawn_blocking(move || inspect_file(&owned))
        .await
        .map_err(|e| AppError::Internal(format!("lecture du CSV interrompue : {e}")))??;
    let matches = store::matching_report(pool, &inspection.signature).await?;
    Ok(CsvInspection {
        inspection,
        saved: matches.found,
        unreadable_mappings: matches.unreadable,
        targets: targets::catalogue(),
    })
}

#[tauri::command]
pub async fn import_source_file(
    source: String,
    path: String,
    app: AppHandle,
    pool: State<'_, SqlitePool>,
    control: State<'_, ImportControl>,
) -> Result<ImportReport, AppError> {
    let emit = |p: ImportProgress| {
        if let Err(e) = app.emit("import-progress", p) {
            log::warn!("[import] progression non emise : {e}");
        }
    };
    import_file_inner(control.inner(), pool.inner(), &source, &path, &emit).await
}

#[tauri::command]
pub async fn import_csv_file(
    path: String,
    config: CsvConfig,
    app: AppHandle,
    pool: State<'_, SqlitePool>,
    control: State<'_, ImportControl>,
) -> Result<ImportReport, AppError> {
    let emit = |p: ImportProgress| {
        if let Err(e) = app.emit("import-progress", p) {
            log::warn!("[import] progression non emise : {e}");
        }
    };
    import_csv_inner(control.inner(), pool.inner(), &path, config, &emit).await
}

#[tauri::command]
pub async fn inspect_csv_file(
    path: String,
    pool: State<'_, SqlitePool>,
) -> Result<CsvInspection, AppError> {
    inspect_csv_inner(pool.inner(), &path).await
}

#[tauri::command]
pub async fn save_csv_mapping(
    name: String,
    signature: String,
    config: CsvConfig,
    pool: State<'_, SqlitePool>,
) -> Result<i64, AppError> {
    store::save(pool.inner(), &name, &signature, &config).await
}

#[tauri::command]
pub async fn delete_csv_mapping(id: i64, pool: State<'_, SqlitePool>) -> Result<bool, AppError> {
    store::remove(pool.inner(), id).await
}

#[tauri::command]
pub fn cancel_import(control: State<'_, ImportControl>) -> bool {
    control.cancel()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_control_is_the_one_of_the_service() {
        let control = ImportControl::default();
        assert!(!control.cancel());
    }
}
