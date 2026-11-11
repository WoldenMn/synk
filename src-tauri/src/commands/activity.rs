use crate::commands::crud::{
    delete_with_audit, record_audit_insert, select_latest, select_range, CrudTable,
};
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::State;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Activity {
    pub id: i64,
    pub date: String,
    pub source: String,
    pub steps: Option<i64>,
    pub distance: Option<f64>,
    pub elevation: Option<f64>,
    pub calories_active: Option<f64>,
    pub calories_total: Option<f64>,
    pub duration_soft: Option<i64>,
    pub duration_moderate: Option<i64>,
    pub duration_intense: Option<i64>,
    pub duration_active: Option<i64>,
    pub hr_average: Option<i64>,
    pub hr_min: Option<i64>,
    pub hr_max: Option<i64>,
    pub hr_zone_0: Option<i64>,
    pub hr_zone_1: Option<i64>,
    pub hr_zone_2: Option<i64>,
    pub hr_zone_3: Option<i64>,
}

impl CrudTable for Activity {
    const TABLE: &'static str = "activity";
    const SELECT_COLS: &'static str =
        "id, date, source, steps, distance, elevation, calories_active, calories_total, \
         duration_soft, duration_moderate, duration_intense, duration_active, \
         hr_average, hr_min, hr_max, hr_zone_0, hr_zone_1, hr_zone_2, hr_zone_3";
    const DATE_COL: &'static str = "date";
}

#[derive(Debug, Deserialize)]
pub struct ManualActivityInput {
    pub date: String,
    pub steps: Option<i64>,
    pub distance: Option<f64>,
    pub calories_active: Option<f64>,
    pub duration_active: Option<i64>,
}

async fn add_manual_activity_inner(
    pool: &SqlitePool,
    data: ManualActivityInput,
) -> Result<i64, AppError> {
    use crate::validation::{check_f64_opt, check_i64_opt};
    check_i64_opt("Pas", data.steps, 0, 200_000)?;
    check_f64_opt("Distance", data.distance, 0.0, 1_000_000.0)?;
    check_f64_opt("Calories actives", data.calories_active, 0.0, 100_000.0)?;
    check_i64_opt("Duree active", data.duration_active, 0, 86_400)?;
    let id = sqlx::query(
        "INSERT INTO activity (date, source, steps, distance, calories_active, duration_active) \
         VALUES (?1, 'manual', ?2, ?3, ?4, ?5)",
    )
    .bind(&data.date)
    .bind(data.steps)
    .bind(data.distance)
    .bind(data.calories_active)
    .bind(data.duration_active)
    .execute(pool)
    .await?
    .last_insert_rowid();
    record_audit_insert(pool, "activity", id).await;
    Ok(id)
}

#[tauri::command]
pub async fn get_activity(
    pool: State<'_, SqlitePool>,
    from: String,
    to: String,
) -> Result<Vec<Activity>, AppError> {
    select_range(pool.inner(), from, to).await
}

#[tauri::command]
pub async fn get_latest_activity(
    pool: State<'_, SqlitePool>,
) -> Result<Option<Activity>, AppError> {
    select_latest(pool.inner()).await
}

#[tauri::command]
pub async fn add_manual_activity(
    pool: State<'_, SqlitePool>,
    data: ManualActivityInput,
) -> Result<i64, AppError> {
    add_manual_activity_inner(pool.inner(), data).await
}

#[tauri::command]
pub async fn delete_activity(pool: State<'_, SqlitePool>, id: i64) -> Result<(), AppError> {
    delete_with_audit(pool.inner(), "activity", id).await
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
    async fn rejects_negative_steps() {
        let p = pool().await;
        let d = ManualActivityInput {
            date: "2026-06-19".into(),
            steps: Some(-10),
            distance: None,
            calories_active: None,
            duration_active: None,
        };
        assert!(add_manual_activity_inner(&p, d).await.is_err());
    }
    #[tokio::test]
    async fn accepts_valid() {
        let p = pool().await;
        let d = ManualActivityInput {
            date: "2026-06-19".into(),
            steps: Some(8000),
            distance: Some(5.5),
            calories_active: Some(300.0),
            duration_active: Some(3600),
        };
        assert!(add_manual_activity_inner(&p, d).await.is_ok());
    }
}
