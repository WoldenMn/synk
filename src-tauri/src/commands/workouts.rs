use crate::commands::crud::{
    delete_with_audit, record_audit_insert, select_latest, select_range, CrudTable,
};
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::State;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Workout {
    pub id: i64,
    pub date: String,
    pub source: String,
    pub start_at: Option<String>,
    pub end_at: Option<String>,
    pub category: Option<i64>,
    pub category_label: Option<String>,
    pub duration: Option<i64>,
    pub pause_duration: Option<i64>,
    pub distance: Option<f64>,
    pub steps: Option<i64>,
    pub elevation: Option<f64>,
    pub calories: Option<f64>,
    pub hr_average: Option<i64>,
    pub hr_min: Option<i64>,
    pub hr_max: Option<i64>,
    pub hr_zone_0: Option<i64>,
    pub hr_zone_1: Option<i64>,
    pub hr_zone_2: Option<i64>,
    pub hr_zone_3: Option<i64>,
    pub spo2_average: Option<f64>,
    pub intensity: Option<i64>,
    pub effort_score: Option<f64>,
    pub pool_laps: Option<i64>,
    pub pool_length: Option<i64>,
}

impl CrudTable for Workout {
    const TABLE: &'static str = "workouts";
    const SELECT_COLS: &'static str =
        "id, date, source, start_at, end_at, category, category_label, duration, \
         pause_duration, distance, steps, elevation, calories, hr_average, hr_min, hr_max, \
         hr_zone_0, hr_zone_1, hr_zone_2, hr_zone_3, spo2_average, intensity, effort_score, \
         pool_laps, pool_length";
    const DATE_COL: &'static str = "date";
    const RANGE_ORDER: &'static str = "DESC";
}

#[derive(Debug, Deserialize)]
pub struct ManualWorkoutInput {
    pub date: String,
    pub category_label: Option<String>,
    pub duration: Option<i64>,
    pub distance: Option<f64>,
    pub steps: Option<i64>,
    pub calories: Option<f64>,
    pub hr_average: Option<i64>,
}

#[tauri::command]
pub async fn get_workouts(
    pool: State<'_, SqlitePool>,
    from: String,
    to: String,
) -> Result<Vec<Workout>, AppError> {
    select_range(pool.inner(), from, to).await
}

#[tauri::command]
pub async fn get_latest_workout(pool: State<'_, SqlitePool>) -> Result<Option<Workout>, AppError> {
    select_latest(pool.inner()).await
}

async fn add_manual_workout_inner(
    pool: &SqlitePool,
    data: ManualWorkoutInput,
) -> Result<i64, AppError> {
    use crate::validation::{check_f64_opt, check_i64_opt};
    check_i64_opt("Duree", data.duration, 0, 86400)?;
    check_f64_opt("Distance", data.distance, 0.0, 1000000.0)?;
    check_f64_opt("Calories", data.calories, 0.0, 100000.0)?;
    check_i64_opt("Frequence cardiaque moyenne", data.hr_average, 0, 300)?;
    let id = sqlx::query(
        "INSERT INTO workouts (date, source, category_label, duration, distance, steps, calories, hr_average) \
         VALUES (?1, 'manual', ?2, ?3, ?4, ?5, ?6, ?7)",
    )
    .bind(&data.date)
    .bind(&data.category_label)
    .bind(data.duration)
    .bind(data.distance)
    .bind(data.steps)
    .bind(data.calories)
    .bind(data.hr_average)
    .execute(pool)
    .await?
    .last_insert_rowid();
    record_audit_insert(pool, "workouts", id).await;
    Ok(id)
}

#[tauri::command]
pub async fn add_manual_workout(
    pool: State<'_, SqlitePool>,
    data: ManualWorkoutInput,
) -> Result<i64, AppError> {
    add_manual_workout_inner(pool.inner(), data).await
}

#[tauri::command]
pub async fn delete_workout(pool: State<'_, SqlitePool>, id: i64) -> Result<(), AppError> {
    delete_with_audit(pool.inner(), "workouts", id).await
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

    fn base() -> ManualWorkoutInput {
        ManualWorkoutInput {
            date: "2026-06-19".into(),
            category_label: Some("Course".into()),
            duration: None,
            distance: None,
            steps: None,
            calories: None,
            hr_average: None,
        }
    }

    #[tokio::test]
    async fn rejects_out_of_range_duration() {
        let p = pool().await;
        let data = ManualWorkoutInput {
            duration: Some(999999),
            ..base()
        };
        assert!(add_manual_workout_inner(&p, data).await.is_err());
    }

    #[tokio::test]
    async fn rejects_out_of_range_hr_average() {
        let p = pool().await;
        let data = ManualWorkoutInput {
            hr_average: Some(9999),
            ..base()
        };
        assert!(add_manual_workout_inner(&p, data).await.is_err());
    }

    #[tokio::test]
    async fn rejects_out_of_range_calories() {
        let p = pool().await;
        let data = ManualWorkoutInput {
            calories: Some(999999.0),
            ..base()
        };
        assert!(add_manual_workout_inner(&p, data).await.is_err());
    }

    #[tokio::test]
    async fn accepts_valid() {
        let p = pool().await;
        let data = ManualWorkoutInput {
            duration: Some(3600),
            distance: Some(8000.0),
            calories: Some(450.0),
            hr_average: Some(140),
            ..base()
        };
        assert!(add_manual_workout_inner(&p, data).await.is_ok());
    }
}
