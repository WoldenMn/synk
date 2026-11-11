use crate::error::AppError;
use crate::utils::dates::day_range;
use crate::validation::{check_date, check_f64, check_f64_opt, check_text, check_text_opt};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::State;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct LabResult {
    pub id: i64,
    pub date: String,
    pub test_name: String,
    pub value: f64,
    pub unit: Option<String>,
    pub reference_min: Option<f64>,
    pub reference_max: Option<f64>,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct AddLabResultInput {
    pub date: String,
    pub test_name: String,
    pub value: f64,
    pub unit: Option<String>,
    pub reference_min: Option<f64>,
    pub reference_max: Option<f64>,
    pub notes: Option<String>,
}

impl AddLabResultInput {
    pub fn validate(&self) -> Result<(), AppError> {
        check_date("Date", &self.date)?;
        check_text("Nom de l'analyse", &self.test_name, 100)?;
        check_f64("Valeur", self.value, -1_000_000.0, 1_000_000.0)?;
        check_text_opt("Unite", self.unit.as_deref(), 20)?;
        check_f64_opt(
            "Reference min",
            self.reference_min,
            -1_000_000.0,
            1_000_000.0,
        )?;
        check_f64_opt(
            "Reference max",
            self.reference_max,
            -1_000_000.0,
            1_000_000.0,
        )?;
        if let (Some(min), Some(max)) = (self.reference_min, self.reference_max) {
            if min > max {
                return Err(AppError::Validation(
                    "Reference min doit etre inferieure ou egale a reference max".into(),
                ));
            }
        }
        check_text_opt("Notes", self.notes.as_deref(), 1_000)
    }
}

#[tauri::command]
pub async fn get_lab_results(
    pool: State<'_, SqlitePool>,
    from: String,
    to: String,
) -> Result<Vec<LabResult>, AppError> {
    lab_results_between(pool.inner(), &from, &to).await
}

pub async fn lab_results_between(
    pool: &SqlitePool,
    from: &str,
    to: &str,
) -> Result<Vec<LabResult>, AppError> {
    let range = day_range(from, to)?;
    let (from_d, to_d) = (range.from, range.before);
    sqlx::query_as::<_, LabResult>(
        "SELECT id, date, test_name, value, unit, reference_min, reference_max, notes, created_at \
         FROM lab_results \
         WHERE date >= ?1 \
         AND date < ?2 \
         ORDER BY date DESC, test_name ASC",
    )
    .bind(&from_d)
    .bind(&to_d)
    .fetch_all(pool)
    .await
    .map_err(AppError::from)
}

#[tauri::command]
pub async fn add_lab_result(
    pool: State<'_, SqlitePool>,
    data: AddLabResultInput,
) -> Result<i64, AppError> {
    data.validate()?;
    let result = sqlx::query(
        "INSERT INTO lab_results (date, test_name, value, unit, reference_min, reference_max, notes) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
    )
    .bind(&data.date)
    .bind(&data.test_name)
    .bind(data.value)
    .bind(&data.unit)
    .bind(data.reference_min)
    .bind(data.reference_max)
    .bind(&data.notes)
    .execute(pool.inner())
    .await?;

    Ok(result.last_insert_rowid())
}

#[tauri::command]
pub async fn delete_lab_result(pool: State<'_, SqlitePool>, id: i64) -> Result<(), AppError> {
    crate::commands::crud::delete_by_id(pool.inner(), "lab_results", "lab_results", id).await
}
