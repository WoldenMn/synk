mod callouts;
mod canvas;
mod display;
mod fetch;
mod figure_page;
mod figures;
mod layout;
mod profile;
mod render;
mod report_lines;
mod sheet;
mod text_metrics;

pub use profile::*;
pub use sheet::*;

use crate::error::AppError;
use sqlx::SqlitePool;
use tauri::State;

use crate::commands::export::shared::resolve_output_dir;
use crate::commands::export::ExportOutcome;
use crate::utils::dates::{date_prefix, day_range};

#[tauri::command]
pub async fn export_pdf(
    app: tauri::AppHandle,
    pool: State<'_, SqlitePool>,
    from: String,
    to: String,
    categories: Vec<String>,
) -> Result<ExportOutcome, AppError> {
    let output_dir = resolve_output_dir(&app)?;
    tokio::fs::create_dir_all(&output_dir).await?;

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let output_path = output_dir.join(format!("synk_rapport_{timestamp}.pdf"));
    let rows = export_pdf_to(pool.inner(), &from, &to, &categories, &output_path).await?;
    Ok(ExportOutcome {
        path: (rows > 0).then(|| output_path.to_string_lossy().into_owned()),
        rows,
    })
}

// `pub` requis : les tests d'integration dans `tests/` sont compiles comme
// des crates separes.
pub async fn export_pdf_to(
    pool: &SqlitePool,
    from: &str,
    to: &str,
    categories: &[String],
    output_path: &std::path::Path,
) -> Result<u64, AppError> {
    let range = day_range(from, to)?;
    let date_from = range.from;
    let date_to = date_prefix(to).to_string();
    let date_before = range.before;

    // Phase 1: fetch async (sqlx, Send-friendly)
    let data = fetch::fetch_pdf_data(pool, &date_from, &date_before, categories).await?;

    // Phase 2: render sync
    let rows = data.row_count();
    if rows == 0 {
        return Ok(0);
    }
    let output = output_path.to_path_buf();
    let categories = categories.to_vec();
    tauri::async_runtime::spawn_blocking(move || {
        render::render_pdf(&data, &output, &date_from, &date_to, &categories)
    })
    .await
    .map_err(|e| AppError::Internal(format!("rendu PDF interrompu : {e}")))??;
    Ok(rows)
}
