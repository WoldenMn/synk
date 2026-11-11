use sqlx::SqlitePool;

use super::mapping::{ActivityRow, SleepRow, WorkoutRow};
use crate::error::AppError;

fn failed(what: &str, e: sqlx::Error) -> AppError {
    AppError::Internal(format!("Google Health : ecriture {what} impossible : {e}"))
}

pub async fn write_sleep(pool: &SqlitePool, rows: &[SleepRow]) -> Result<usize, AppError> {
    let mut tx = pool.begin().await?;
    for r in rows {
        sqlx::query(
            "INSERT INTO sleep_records (date, source, external_id, start_at, end_at, duration_total, \
             duration_deep, duration_light, duration_rem, duration_awake, duration_to_sleep, \
             total_timeinbed, sleep_efficiency, rr_average) \
             VALUES (?1, 'google_health', ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13) \
             ON CONFLICT(date, source) DO UPDATE SET \
             external_id = excluded.external_id, \
             start_at = excluded.start_at, \
             end_at = excluded.end_at, \
             duration_total = excluded.duration_total, \
             duration_deep = excluded.duration_deep, \
             duration_light = excluded.duration_light, \
             duration_rem = excluded.duration_rem, \
             duration_awake = excluded.duration_awake, \
             duration_to_sleep = excluded.duration_to_sleep, \
             total_timeinbed = excluded.total_timeinbed, \
             sleep_efficiency = excluded.sleep_efficiency, \
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
        .bind(r.sleep_efficiency)
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
             calories_total) VALUES (?1, 'google_health', ?2, ?3, ?4, ?5, ?6) \
             ON CONFLICT(date, source) DO UPDATE SET \
             external_id = excluded.external_id, \
             steps = COALESCE(excluded.steps, activity.steps), \
             distance = COALESCE(excluded.distance, activity.distance), \
             calories_active = COALESCE(excluded.calories_active, activity.calories_active), \
             calories_total = COALESCE(excluded.calories_total, activity.calories_total)",
        )
        .bind(&r.date)
        .bind(&r.external_id)
        .bind(r.steps)
        .bind(r.distance)
        .bind(r.calories_active)
        .bind(r.calories_total)
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
             duration, distance, calories) VALUES (?1, 'google_health', ?2, ?3, ?4, ?5, ?6, ?7, ?8) \
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
