use sqlx::SqlitePool;

use super::mapping::{ActivityRow, SleepRow, WorkoutRow};
use crate::error::AppError;

fn failed(what: &str, e: sqlx::Error) -> AppError {
    AppError::Internal(format!("Oura : ecriture {what} impossible : {e}"))
}

pub async fn write_sleep(pool: &SqlitePool, rows: &[SleepRow]) -> Result<usize, AppError> {
    let mut tx = pool.begin().await?;
    for r in rows {
        sqlx::query(
            "INSERT INTO sleep_records (date, source, external_id, start_at, end_at, duration_total, \
             duration_deep, duration_light, duration_rem, duration_awake, duration_to_sleep, \
             total_timeinbed, sleep_score, sleep_efficiency, hr_average, hr_min, rr_average) \
             VALUES (?1, 'oura', ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16) \
             ON CONFLICT(date, source) DO UPDATE SET \
             external_id = COALESCE(excluded.external_id, sleep_records.external_id), \
             start_at = COALESCE(excluded.start_at, sleep_records.start_at), \
             end_at = COALESCE(excluded.end_at, sleep_records.end_at), \
             duration_total = COALESCE(excluded.duration_total, sleep_records.duration_total), \
             duration_deep = COALESCE(excluded.duration_deep, sleep_records.duration_deep), \
             duration_light = COALESCE(excluded.duration_light, sleep_records.duration_light), \
             duration_rem = COALESCE(excluded.duration_rem, sleep_records.duration_rem), \
             duration_awake = COALESCE(excluded.duration_awake, sleep_records.duration_awake), \
             duration_to_sleep = COALESCE(excluded.duration_to_sleep, sleep_records.duration_to_sleep), \
             total_timeinbed = COALESCE(excluded.total_timeinbed, sleep_records.total_timeinbed), \
             sleep_score = COALESCE(excluded.sleep_score, sleep_records.sleep_score), \
             sleep_efficiency = COALESCE(excluded.sleep_efficiency, sleep_records.sleep_efficiency), \
             hr_average = COALESCE(excluded.hr_average, sleep_records.hr_average), \
             hr_min = COALESCE(excluded.hr_min, sleep_records.hr_min), \
             rr_average = COALESCE(excluded.rr_average, sleep_records.rr_average)",
        )
        .bind(&r.date)
        .bind(&r.external_id)
        .bind(&r.start_at)
        .bind(&r.end_at)
        .bind(r.duration_total)
        .bind(r.duration_deep)
        .bind(r.duration_light)
        .bind(r.duration_rem)
        .bind(r.duration_awake)
        .bind(r.duration_to_sleep)
        .bind(r.total_timeinbed)
        .bind(r.sleep_score)
        .bind(r.sleep_efficiency)
        .bind(r.hr_average)
        .bind(r.hr_min)
        .bind(r.rr_average)
        .execute(&mut *tx)
        .await
        .map_err(|e| failed("du sommeil", e))?;
    }
    tx.commit().await?;
    Ok(rows.len())
}

pub async fn write_activity(pool: &SqlitePool, rows: &[ActivityRow]) -> Result<usize, AppError> {
    let mut tx = pool.begin().await?;
    for r in rows {
        sqlx::query(
            "INSERT INTO activity (date, source, external_id, steps, distance, calories_active, \
             calories_total, duration_soft, duration_moderate, duration_intense, duration_active) \
             VALUES (?1, 'oura', ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10) \
             ON CONFLICT(date, source) DO UPDATE SET \
             external_id = excluded.external_id, steps = excluded.steps, \
             distance = excluded.distance, calories_active = excluded.calories_active, \
             calories_total = excluded.calories_total, duration_soft = excluded.duration_soft, \
             duration_moderate = excluded.duration_moderate, \
             duration_intense = excluded.duration_intense, \
             duration_active = excluded.duration_active",
        )
        .bind(&r.date)
        .bind(&r.external_id)
        .bind(r.steps)
        .bind(r.distance)
        .bind(r.calories_active)
        .bind(r.calories_total)
        .bind(r.duration_soft)
        .bind(r.duration_moderate)
        .bind(r.duration_intense)
        .bind(r.duration_active)
        .execute(&mut *tx)
        .await
        .map_err(|e| failed("de l activite", e))?;
    }
    tx.commit().await?;
    Ok(rows.len())
}

pub async fn write_workouts(pool: &SqlitePool, rows: &[WorkoutRow]) -> Result<usize, AppError> {
    let mut tx = pool.begin().await?;
    for r in rows {
        sqlx::query(
            "INSERT INTO workouts (date, source, external_id, start_at, end_at, category_label, \
             duration, distance, calories) VALUES (?1, 'oura', ?2, ?3, ?4, ?5, ?6, ?7, ?8) \
             ON CONFLICT(source, external_id) DO UPDATE SET \
             date = excluded.date, start_at = excluded.start_at, end_at = excluded.end_at, \
             category_label = excluded.category_label, duration = excluded.duration, \
             distance = excluded.distance, calories = excluded.calories",
        )
        .bind(&r.date)
        .bind(&r.external_id)
        .bind(&r.start_at)
        .bind(&r.end_at)
        .bind(&r.category_label)
        .bind(r.duration)
        .bind(r.distance)
        .bind(r.calories)
        .execute(&mut *tx)
        .await
        .map_err(|e| failed("des entrainements", e))?;
    }
    tx.commit().await?;
    Ok(rows.len())
}
