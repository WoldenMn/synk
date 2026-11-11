use crate::error::AppError;
use crate::utils::dates::day_range;
use crate::validation::{check_date, check_text, check_text_opt};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::State;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Medication {
    pub id: i64,
    pub name: String,
    pub dosage: Option<String>,
    pub frequency: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub notes: Option<String>,
    pub active: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct MedicationLog {
    pub id: i64,
    pub medication_id: i64,
    pub taken_at: String,
    pub skipped: bool,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct AddMedicationInput {
    pub name: String,
    pub dosage: Option<String>,
    pub frequency: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMedicationInput {
    pub id: i64,
    pub name: String,
    pub dosage: Option<String>,
    pub frequency: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub notes: Option<String>,
    pub active: bool,
}

#[derive(Debug, Deserialize)]
pub struct LogMedicationInput {
    pub medication_id: i64,
    pub taken_at: String,
    pub skipped: bool,
    pub notes: Option<String>,
}

fn check_medication(
    name: &str,
    dosage: Option<&str>,
    frequency: Option<&str>,
    start_date: Option<&str>,
    end_date: Option<&str>,
    notes: Option<&str>,
) -> Result<(), AppError> {
    check_text("Nom", name, 100)?;
    check_text_opt("Dosage", dosage, 100)?;
    check_text_opt("Frequence", frequency, 100)?;
    if let Some(d) = start_date {
        check_date("Debut", d)?;
    }
    if let Some(d) = end_date {
        check_date("Fin", d)?;
    }
    if let (Some(start), Some(end)) = (start_date, end_date) {
        if end < start {
            return Err(AppError::Validation("Fin doit suivre le debut".into()));
        }
    }
    check_text_opt("Notes", notes, 1_000)
}

impl AddMedicationInput {
    pub fn validate(&self) -> Result<(), AppError> {
        check_medication(
            &self.name,
            self.dosage.as_deref(),
            self.frequency.as_deref(),
            self.start_date.as_deref(),
            self.end_date.as_deref(),
            self.notes.as_deref(),
        )
    }
}

impl UpdateMedicationInput {
    pub fn validate(&self) -> Result<(), AppError> {
        check_medication(
            &self.name,
            self.dosage.as_deref(),
            self.frequency.as_deref(),
            self.start_date.as_deref(),
            self.end_date.as_deref(),
            self.notes.as_deref(),
        )
    }
}

impl LogMedicationInput {
    pub fn validate(&self) -> Result<(), AppError> {
        chrono::DateTime::parse_from_rfc3339(&self.taken_at)
            .map_err(|_| AppError::Validation("Heure de prise invalide".into()))?;
        check_text_opt("Notes", self.notes.as_deref(), 1_000)
    }
}

#[tauri::command]
pub async fn get_medications(pool: State<'_, SqlitePool>) -> Result<Vec<Medication>, AppError> {
    sqlx::query_as::<_, Medication>(
        "SELECT id, name, dosage, frequency, start_date, end_date, notes, active, created_at \
         FROM medications ORDER BY active DESC, name ASC",
    )
    .fetch_all(pool.inner())
    .await
    .map_err(AppError::from)
}

#[tauri::command]
pub async fn add_medication(
    pool: State<'_, SqlitePool>,
    data: AddMedicationInput,
) -> Result<i64, AppError> {
    data.validate()?;
    let result = sqlx::query(
        "INSERT INTO medications (name, dosage, frequency, start_date, end_date, notes) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    )
    .bind(&data.name)
    .bind(&data.dosage)
    .bind(&data.frequency)
    .bind(&data.start_date)
    .bind(&data.end_date)
    .bind(&data.notes)
    .execute(pool.inner())
    .await?;

    Ok(result.last_insert_rowid())
}

#[tauri::command]
pub async fn update_medication(
    pool: State<'_, SqlitePool>,
    data: UpdateMedicationInput,
) -> Result<(), AppError> {
    data.validate()?;
    sqlx::query(
        "UPDATE medications SET name = ?1, dosage = ?2, frequency = ?3, \
         start_date = ?4, end_date = ?5, notes = ?6, active = ?7 WHERE id = ?8",
    )
    .bind(&data.name)
    .bind(&data.dosage)
    .bind(&data.frequency)
    .bind(&data.start_date)
    .bind(&data.end_date)
    .bind(&data.notes)
    .bind(data.active)
    .bind(data.id)
    .execute(pool.inner())
    .await?;

    Ok(())
}

pub async fn delete_medication_inner(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
    let deleted = sqlx::query("DELETE FROM medications WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    if deleted == 0 {
        return Err(AppError::NotFound {
            entity: "medications",
            id,
        });
    }
    crate::services::audit::record_audit_or_warn(pool, "delete", "medications", Some(id), None)
        .await;
    Ok(())
}

#[tauri::command]
pub async fn delete_medication(pool: State<'_, SqlitePool>, id: i64) -> Result<(), AppError> {
    delete_medication_inner(pool.inner(), id).await
}

#[tauri::command]
pub async fn log_medication_taken(
    pool: State<'_, SqlitePool>,
    data: LogMedicationInput,
) -> Result<i64, AppError> {
    data.validate()?;
    let result = sqlx::query(
        "INSERT INTO medication_logs (medication_id, taken_at, skipped, notes) \
         VALUES (?1, ?2, ?3, ?4)",
    )
    .bind(data.medication_id)
    .bind(&data.taken_at)
    .bind(data.skipped)
    .bind(&data.notes)
    .execute(pool.inner())
    .await?;

    Ok(result.last_insert_rowid())
}

#[tauri::command]
pub async fn get_medication_logs(
    pool: State<'_, SqlitePool>,
    medication_id: i64,
    from: String,
    to: String,
) -> Result<Vec<MedicationLog>, AppError> {
    medication_logs_between(pool.inner(), medication_id, &from, &to).await
}

pub async fn medication_logs_between(
    pool: &SqlitePool,
    medication_id: i64,
    from: &str,
    to: &str,
) -> Result<Vec<MedicationLog>, AppError> {
    let range = day_range(from, to)?;
    let (from_d, to_d) = (range.from, range.before);
    sqlx::query_as::<_, MedicationLog>(
        "SELECT id, medication_id, taken_at, skipped, notes, created_at \
         FROM medication_logs \
         WHERE medication_id = ?1 \
         AND taken_at >= ?2 \
         AND taken_at < ?3 \
         ORDER BY taken_at DESC",
    )
    .bind(medication_id)
    .bind(&from_d)
    .bind(&to_d)
    .fetch_all(pool)
    .await
    .map_err(AppError::from)
}
