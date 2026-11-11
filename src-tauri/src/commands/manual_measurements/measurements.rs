use crate::error::AppError;
use crate::utils::dates::day_range;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::State;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ManualMeasurement {
    pub id: i64,
    pub measurement_type_id: i64,
    pub measured_at: String,
    pub value: f64,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddManualMeasurementInput {
    pub measurement_type_id: i64,
    pub measured_at: String,
    pub value: f64,
    pub notes: Option<String>,
}

#[tauri::command]
pub async fn get_manual_measurements(
    pool: State<'_, SqlitePool>,
    type_id: i64,
    from: String,
    to: String,
) -> Result<Vec<ManualMeasurement>, AppError> {
    manual_measurements_between(pool.inner(), type_id, &from, &to).await
}

pub async fn manual_measurements_between(
    pool: &SqlitePool,
    type_id: i64,
    from: &str,
    to: &str,
) -> Result<Vec<ManualMeasurement>, AppError> {
    let range = day_range(from, to)?;
    let (from_d, to_d) = (range.from, range.before);
    sqlx::query_as::<_, ManualMeasurement>(
        "SELECT id, measurement_type_id, measured_at, value, notes \
         FROM manual_measurements \
         WHERE measurement_type_id = ?1 AND measured_at >= ?2 AND measured_at < ?3 \
         ORDER BY measured_at ASC",
    )
    .bind(type_id)
    .bind(&from_d)
    .bind(&to_d)
    .fetch_all(pool)
    .await
    .map_err(AppError::from)
}

async fn add_manual_measurement_inner(
    pool: &SqlitePool,
    data: AddManualMeasurementInput,
) -> Result<i64, AppError> {
    crate::validation::check_f64("Valeur", data.value, -1_000_000.0, 1_000_000.0)?;
    let result = sqlx::query(
        "INSERT INTO manual_measurements_legacy (measurement_type_id, measured_at, value, notes) \
         VALUES (?1, ?2, ?3, ?4)",
    )
    .bind(data.measurement_type_id)
    .bind(&data.measured_at)
    .bind(data.value)
    .bind(&data.notes)
    .execute(pool)
    .await?;
    Ok(result.last_insert_rowid())
}

#[tauri::command]
pub async fn add_manual_measurement(
    pool: State<'_, SqlitePool>,
    data: AddManualMeasurementInput,
) -> Result<i64, AppError> {
    add_manual_measurement_inner(pool.inner(), data).await
}

#[cfg(test)]
mod validation_tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;
    async fn pool() -> SqlitePool {
        let p = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(":memory:")
            .await
            .unwrap();
        crate::db::bootstrap_schema(&p).await.unwrap();
        p
    }
    #[tokio::test]
    async fn rejects_non_finite_value() {
        let p = pool().await;
        let d = AddManualMeasurementInput {
            measurement_type_id: 1,
            measured_at: "2026-06-19T08:00:00Z".into(),
            value: f64::NAN,
            notes: None,
        };
        assert!(add_manual_measurement_inner(&p, d).await.is_err());
    }
}

#[tauri::command]
pub async fn delete_manual_measurement(
    pool: State<'_, SqlitePool>,
    id: i64,
) -> Result<(), AppError> {
    crate::commands::crud::delete_by_id(
        pool.inner(),
        "manual_measurements_legacy",
        "manual_measurements",
        id,
    )
    .await
}
