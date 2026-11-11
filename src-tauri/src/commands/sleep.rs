use crate::commands::crud::{
    delete_with_audit, record_audit_insert, select_latest, select_range, CrudTable,
};
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::State;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SleepRecord {
    pub id: i64,
    pub date: String,
    pub source: String,
    pub start_at: Option<String>,
    pub end_at: Option<String>,
    pub duration_total: Option<i64>,
    pub duration_deep: Option<i64>,
    pub duration_light: Option<i64>,
    pub duration_rem: Option<i64>,
    pub duration_awake: Option<i64>,
    pub duration_to_sleep: Option<i64>,
    pub duration_to_wakeup: Option<i64>,
    pub sleep_score: Option<i64>,
    pub hr_average: Option<i64>,
    pub hr_min: Option<i64>,
    pub hr_max: Option<i64>,
    pub rr_average: Option<f64>,
    pub snoring: Option<i64>,
    pub snoring_count: Option<i64>,
    pub sleep_efficiency: Option<f64>,
    pub nb_rem_episodes: Option<i64>,
    pub out_of_bed_count: Option<i64>,
    pub waso: Option<i64>,
    pub total_timeinbed: Option<i64>,
    pub apnea_hypopnea_index: Option<f64>,
    pub breathing_disturbances_intensity: Option<f64>,
    pub asleepduration: Option<i64>,
    pub night_events: Option<i64>,
}

impl CrudTable for SleepRecord {
    const TABLE: &'static str = "sleep_records";
    const SELECT_COLS: &'static str =
        "id, date, source, start_at, end_at, duration_total, duration_deep, duration_light, \
        duration_rem, duration_awake, duration_to_sleep, duration_to_wakeup, sleep_score, \
        hr_average, hr_min, hr_max, rr_average, snoring, snoring_count, \
        sleep_efficiency, nb_rem_episodes, out_of_bed_count, waso, total_timeinbed, \
        apnea_hypopnea_index, breathing_disturbances_intensity, asleepduration, night_events";
    const DATE_COL: &'static str = "date";
}

#[derive(Debug, Deserialize)]
pub struct ManualSleepInput {
    pub date: String,
    pub start_at: Option<String>,
    pub end_at: Option<String>,
    pub duration_total: Option<i64>,
    pub sleep_score: Option<i64>,
}

async fn add_manual_sleep_inner(
    pool: &SqlitePool,
    data: ManualSleepInput,
) -> Result<i64, AppError> {
    use crate::validation::check_i64_opt;
    check_i64_opt("Duree de sommeil", data.duration_total, 0, 86_400)?;
    check_i64_opt("Score de sommeil", data.sleep_score, 0, 100)?;
    let id = sqlx::query(
        "INSERT INTO sleep_records (date, source, start_at, end_at, duration_total, sleep_score) \
         VALUES (?1, 'manual', ?2, ?3, ?4, ?5)",
    )
    .bind(&data.date)
    .bind(&data.start_at)
    .bind(&data.end_at)
    .bind(data.duration_total)
    .bind(data.sleep_score)
    .execute(pool)
    .await?
    .last_insert_rowid();
    record_audit_insert(pool, "sleep_records", id).await;
    Ok(id)
}

#[tauri::command]
pub async fn get_sleep_records(
    pool: State<'_, SqlitePool>,
    from: String,
    to: String,
) -> Result<Vec<SleepRecord>, AppError> {
    select_range(pool.inner(), from, to).await
}

#[tauri::command]
pub async fn get_latest_sleep(
    pool: State<'_, SqlitePool>,
) -> Result<Option<SleepRecord>, AppError> {
    select_latest(pool.inner()).await
}

#[tauri::command]
pub async fn add_manual_sleep(
    pool: State<'_, SqlitePool>,
    data: ManualSleepInput,
) -> Result<i64, AppError> {
    add_manual_sleep_inner(pool.inner(), data).await
}

#[tauri::command]
pub async fn delete_sleep_record(pool: State<'_, SqlitePool>, id: i64) -> Result<(), AppError> {
    delete_with_audit(pool.inner(), "sleep_records", id).await
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
    async fn rejects_score_over_100() {
        let p = pool().await;
        let d = ManualSleepInput {
            date: "2026-06-19".into(),
            start_at: None,
            end_at: None,
            duration_total: Some(28800),
            sleep_score: Some(150),
        };
        assert!(add_manual_sleep_inner(&p, d).await.is_err());
    }
    #[tokio::test]
    async fn rejects_negative_duration() {
        let p = pool().await;
        let d = ManualSleepInput {
            date: "2026-06-19".into(),
            start_at: None,
            end_at: None,
            duration_total: Some(-1),
            sleep_score: Some(80),
        };
        assert!(add_manual_sleep_inner(&p, d).await.is_err());
    }
    #[tokio::test]
    async fn accepts_valid() {
        let p = pool().await;
        let d = ManualSleepInput {
            date: "2026-06-19".into(),
            start_at: None,
            end_at: None,
            duration_total: Some(28800),
            sleep_score: Some(80),
        };
        assert!(add_manual_sleep_inner(&p, d).await.is_ok());
    }
}
