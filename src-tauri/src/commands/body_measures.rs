use crate::commands::crud::{
    delete_manual, record_audit_insert, select_latest, select_range, CrudTable,
};
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::State;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct BodyMeasure {
    pub id: i64,
    pub measured_at: String,
    pub source: String,
    pub weight: Option<f64>,
    pub fat_ratio: Option<f64>,
    pub fat_mass: Option<f64>,
    pub fat_free_mass: Option<f64>,
    pub muscle_mass: Option<f64>,
    pub bone_mass: Option<f64>,
    pub hydration: Option<f64>,
    pub bmi: Option<f64>,
}

impl CrudTable for BodyMeasure {
    const TABLE: &'static str = "body_measures";
    const SELECT_COLS: &'static str =
        "id, measured_at, source, weight, fat_ratio, fat_mass, fat_free_mass, muscle_mass, bone_mass, hydration, bmi";
    const DATE_COL: &'static str = "measured_at";
}

#[derive(Debug, Deserialize)]
pub struct ManualBodyMeasureInput {
    pub measured_at: String,
    pub weight: Option<f64>,
    pub fat_ratio: Option<f64>,
    pub fat_mass: Option<f64>,
    pub muscle_mass: Option<f64>,
    pub bone_mass: Option<f64>,
    pub hydration: Option<f64>,
}

async fn add_manual_body_measure_inner(
    pool: &SqlitePool,
    data: ManualBodyMeasureInput,
) -> Result<i64, AppError> {
    use crate::validation::check_f64_opt;
    check_f64_opt("Poids", data.weight, 0.0, 500.0)?;
    check_f64_opt("Masse grasse (%)", data.fat_ratio, 0.0, 100.0)?;
    check_f64_opt("Masse grasse (kg)", data.fat_mass, 0.0, 500.0)?;
    check_f64_opt("Masse musculaire", data.muscle_mass, 0.0, 500.0)?;
    check_f64_opt("Masse osseuse", data.bone_mass, 0.0, 100.0)?;
    check_f64_opt("Hydratation (kg)", data.hydration, 0.0, 300.0)?;
    let id = sqlx::query(
        "INSERT INTO body_measures_legacy (measured_at, source, weight, fat_ratio, fat_mass, muscle_mass, bone_mass, hydration) \
         VALUES (?1, 'manual', ?2, ?3, ?4, ?5, ?6, ?7)",
    )
    .bind(&data.measured_at)
    .bind(data.weight)
    .bind(data.fat_ratio)
    .bind(data.fat_mass)
    .bind(data.muscle_mass)
    .bind(data.bone_mass)
    .bind(data.hydration)
    .execute(pool)
    .await?
    .last_insert_rowid();
    record_audit_insert(pool, "body_measures", id).await;
    Ok(id)
}

async fn delete_body_measure_inner(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
    delete_manual(pool, "body_measures_legacy", "body_measures", id).await
}

#[tauri::command]
pub async fn get_body_measures(
    pool: State<'_, SqlitePool>,
    from: String,
    to: String,
) -> Result<Vec<BodyMeasure>, AppError> {
    select_range(pool.inner(), from, to).await
}

#[tauri::command]
pub async fn get_latest_body_measure(
    pool: State<'_, SqlitePool>,
) -> Result<Option<BodyMeasure>, AppError> {
    select_latest(pool.inner()).await
}

#[tauri::command]
pub async fn add_manual_body_measure(
    pool: State<'_, SqlitePool>,
    data: ManualBodyMeasureInput,
) -> Result<i64, AppError> {
    add_manual_body_measure_inner(pool.inner(), data).await
}

#[tauri::command]
pub async fn delete_body_measure(pool: State<'_, SqlitePool>, id: i64) -> Result<(), AppError> {
    delete_body_measure_inner(pool.inner(), id).await
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
    async fn rejects_out_of_range_fat_ratio() {
        let p = pool().await;
        let data = ManualBodyMeasureInput {
            measured_at: "2026-06-19T08:00:00Z".into(),
            weight: Some(70.0),
            fat_ratio: Some(250.0),
            fat_mass: None,
            muscle_mass: None,
            bone_mass: None,
            hydration: None,
        };
        assert!(add_manual_body_measure_inner(&p, data).await.is_err());
    }

    #[tokio::test]
    async fn rejects_negative_weight() {
        let p = pool().await;
        let data = ManualBodyMeasureInput {
            measured_at: "2026-06-19T08:00:00Z".into(),
            weight: Some(-5.0),
            fat_ratio: None,
            fat_mass: None,
            muscle_mass: None,
            bone_mass: None,
            hydration: None,
        };
        assert!(add_manual_body_measure_inner(&p, data).await.is_err());
    }

    #[tokio::test]
    async fn accepts_valid() {
        let p = pool().await;
        let data = ManualBodyMeasureInput {
            measured_at: "2026-06-19T08:00:00Z".into(),
            weight: Some(72.5),
            fat_ratio: Some(18.0),
            fat_mass: Some(13.0),
            muscle_mass: Some(55.0),
            bone_mass: Some(3.0),
            hydration: Some(55.0),
        };
        assert!(add_manual_body_measure_inner(&p, data).await.is_ok());
    }
}
