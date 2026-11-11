use crate::error::AppError;
use crate::utils::dates::day_range;
use csv::WriterBuilder;
use sqlx::SqlitePool;
use std::path::Path;
use tauri::State;

mod body;
mod daily;

pub(super) fn text_cell(value: &str) -> std::borrow::Cow<'_, str> {
    match value.chars().next() {
        Some('=' | '+' | '-' | '@' | '\t' | '\r') => format!("'{value}").into(),
        _ => value.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::text_cell;

    #[test]
    fn a_formula_trigger_is_prefixed() {
        for risky in ["=1+1", "+1", "-1+2", "@x", "\tx", "\rx"] {
            assert_eq!(text_cell(risky), format!("'{risky}"));
        }
    }

    #[test]
    fn ordinary_text_is_left_alone() {
        for fine in ["Course", "Tour de bras", "", "a=b", "5 km"] {
            assert_eq!(text_cell(fine), fine);
        }
    }
}

use super::shared::resolve_output_dir;
use crate::services::backup::with_suffix;

#[derive(Debug, serde::Serialize)]
pub struct ExportOutcome {
    pub path: Option<String>,
    pub rows: u64,
}

#[tauri::command]
pub async fn export_csv(
    app: tauri::AppHandle,
    pool: State<'_, SqlitePool>,
    from: String,
    to: String,
    categories: Vec<String>,
) -> Result<ExportOutcome, AppError> {
    let output_dir = resolve_output_dir(&app)?;
    tokio::fs::create_dir_all(&output_dir).await?;

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let output_path = output_dir.join(format!("synk_export_{timestamp}.csv"));
    let rows = export_csv_inner(pool.inner(), &from, &to, &categories, &output_path).await?;
    Ok(ExportOutcome {
        path: (rows > 0).then(|| output_path.to_string_lossy().into_owned()),
        rows,
    })
}

pub async fn export_csv_inner(
    pool: &SqlitePool,
    from: &str,
    to: &str,
    categories: &[String],
    output_path: &Path,
) -> Result<u64, AppError> {
    let partial = with_suffix(output_path, ".partial");
    let written = write_csv(pool, from, to, categories, &partial).await;
    match written {
        Ok(rows) if rows > 0 => tokio::fs::rename(&partial, output_path)
            .await
            .map(|()| rows)
            .map_err(|e| {
                AppError::Internal(format!(
                    "renommage de {} impossible ({e})",
                    partial.display()
                ))
            }),
        Ok(rows) => discard_partial(&partial).await.map(|()| rows),
        Err(e) => match discard_partial(&partial).await {
            Ok(()) => Err(e),
            Err(rm) => Err(AppError::Internal(format!("{e} ; {rm}"))),
        },
    }
}

async fn discard_partial(partial: &Path) -> Result<(), AppError> {
    match tokio::fs::remove_file(partial).await {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(AppError::Internal(format!(
            "suppression de {} impossible ({e})",
            partial.display()
        ))),
    }
}

async fn write_csv(
    pool: &SqlitePool,
    from: &str,
    to: &str,
    categories: &[String],
    partial: &Path,
) -> Result<u64, AppError> {
    let range = day_range(from, to)?;
    let (date_from, date_before) = (range.from.as_str(), range.before.as_str());
    let mut wtr = WriterBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_writer(Vec::new());
    let mut total: u64 = 0;

    for category in categories {
        match category.as_str() {
            "body_measures" => {
                total += body::write_body_measures(&mut wtr, pool, date_from, date_before).await?
            }
            "blood_pressure" => {
                total += body::write_blood_pressure(&mut wtr, pool, date_from, date_before).await?
            }
            "sleep" => total += daily::write_sleep(&mut wtr, pool, date_from, date_before).await?,
            "activity" => {
                total += daily::write_activity(&mut wtr, pool, date_from, date_before).await?
            }
            "measurements" => {
                total += body::write_measurements(&mut wtr, pool, date_from, date_before).await?
            }
            "workouts" => {
                total += daily::write_workouts(&mut wtr, pool, date_from, date_before).await?
            }
            "heart" => total += daily::write_heart(&mut wtr, pool, date_from, date_before).await?,
            _ => {}
        }
    }

    let bytes = wtr
        .into_inner()
        .map_err(|e| AppError::Internal(format!("ecriture du CSV impossible ({e})")))?;
    tokio::fs::write(partial, bytes).await?;
    Ok(total)
}
