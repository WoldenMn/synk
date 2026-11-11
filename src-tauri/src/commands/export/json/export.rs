use crate::error::AppError;
use sqlx::SqlitePool;
use tauri::State;

use super::super::shared::resolve_output_dir;

pub async fn export_json_inner(
    pool: &SqlitePool,
    output_path: &std::path::Path,
) -> Result<(), AppError> {
    let json = super::engine::export_value(pool).await?;
    let content = serde_json::to_string_pretty(&json)?;
    tokio::fs::write(output_path, content).await.map_err(|e| {
        AppError::Internal(format!(
            "ecriture de {} impossible ({e})",
            output_path.display()
        ))
    })
}

#[tauri::command]
pub async fn export_json(
    app: tauri::AppHandle,
    pool: State<'_, SqlitePool>,
) -> Result<String, AppError> {
    let output_dir = resolve_output_dir(&app)?;
    tokio::fs::create_dir_all(&output_dir).await?;

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let output_path = output_dir.join(format!("synk_export_{timestamp}.json"));
    export_json_inner(pool.inner(), &output_path).await?;
    Ok(output_path.to_string_lossy().into_owned())
}

pub(super) fn row_to_json(row: &sqlx::sqlite::SqliteRow) -> serde_json::Value {
    use sqlx::Column;
    use sqlx::Row;
    let mut map = serde_json::Map::new();
    for col in row.columns() {
        let name = col.name();
        let raw = row
            .try_get_raw(name)
            .expect("colonne issue de row.columns()");
        if sqlx::ValueRef::is_null(&raw) {
            map.insert(name.to_string(), serde_json::Value::Null);
            continue;
        }
        let val: serde_json::Value = if let Ok(v) = row.try_get::<i64, _>(name) {
            serde_json::Value::Number(v.into())
        } else if let Ok(v) = row.try_get::<f64, _>(name) {
            serde_json::Number::from_f64(v)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null)
        } else if let Ok(v) = row.try_get::<String, _>(name) {
            serde_json::Value::String(v)
        } else {
            serde_json::Value::Null
        };
        map.insert(name.to_string(), val);
    }
    serde_json::Value::Object(map)
}
