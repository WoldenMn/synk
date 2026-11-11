use crate::commands::crud::{
    delete_with_audit, record_audit_insert, select_latest, select_range, CrudTable,
};
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::State;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct BloodPressure {
    pub id: i64,
    pub measured_at: String,
    pub source: String,
    pub systolic: i64,
    pub diastolic: i64,
    pub heart_rate: Option<i64>,
}

impl CrudTable for BloodPressure {
    const TABLE: &'static str = "blood_pressure";
    const SELECT_COLS: &'static str = "id, measured_at, source, systolic, diastolic, heart_rate";
    const DATE_COL: &'static str = "measured_at";
}

#[derive(Debug, Deserialize)]
pub struct ManualBloodPressureInput {
    pub measured_at: String,
    pub systolic: i64,
    pub diastolic: i64,
    pub heart_rate: Option<i64>,
}

async fn add_manual_blood_pressure_inner(
    pool: &SqlitePool,
    data: ManualBloodPressureInput,
) -> Result<i64, AppError> {
    if !(0..=300).contains(&data.systolic) {
        return Err(AppError::Validation(
            "Systolique doit etre dans [0, 300]".into(),
        ));
    }
    if !(0..=200).contains(&data.diastolic) {
        return Err(AppError::Validation(
            "Diastolique doit etre dans [0, 200]".into(),
        ));
    }
    crate::validation::check_i64_opt("Frequence cardiaque", data.heart_rate, 0, 300)?;
    let id = sqlx::query(
        "INSERT INTO blood_pressure (measured_at, source, systolic, diastolic, heart_rate) \
         VALUES (?1, 'manual', ?2, ?3, ?4)",
    )
    .bind(&data.measured_at)
    .bind(data.systolic)
    .bind(data.diastolic)
    .bind(data.heart_rate)
    .execute(pool)
    .await?
    .last_insert_rowid();
    record_audit_insert(pool, "blood_pressure", id).await;
    Ok(id)
}

#[tauri::command]
pub async fn get_blood_pressure(
    pool: State<'_, SqlitePool>,
    from: String,
    to: String,
) -> Result<Vec<BloodPressure>, AppError> {
    select_range(pool.inner(), from, to).await
}

#[tauri::command]
pub async fn get_latest_blood_pressure(
    pool: State<'_, SqlitePool>,
) -> Result<Option<BloodPressure>, AppError> {
    select_latest(pool.inner()).await
}

#[tauri::command]
pub async fn add_manual_blood_pressure(
    pool: State<'_, SqlitePool>,
    data: ManualBloodPressureInput,
) -> Result<i64, AppError> {
    add_manual_blood_pressure_inner(pool.inner(), data).await
}

#[tauri::command]
pub async fn delete_blood_pressure(pool: State<'_, SqlitePool>, id: i64) -> Result<(), AppError> {
    delete_with_audit(pool.inner(), "blood_pressure", id).await
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
    async fn rejects_out_of_range_heart_rate() {
        let p = pool().await;
        let data = ManualBloodPressureInput {
            measured_at: "2026-06-19T08:00:00Z".into(),
            systolic: 120,
            diastolic: 80,
            heart_rate: Some(9999),
        };
        assert!(add_manual_blood_pressure_inner(&p, data).await.is_err());
    }

    #[tokio::test]
    async fn accepts_valid_heart_rate() {
        let p = pool().await;
        let data = ManualBloodPressureInput {
            measured_at: "2026-06-19T08:00:00Z".into(),
            systolic: 120,
            diastolic: 80,
            heart_rate: Some(60),
        };
        assert!(add_manual_blood_pressure_inner(&p, data).await.is_ok());
    }
}
