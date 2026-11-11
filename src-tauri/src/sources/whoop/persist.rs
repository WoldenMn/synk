use sqlx::SqlitePool;

use super::mapping::{ActivityRow, SleepRow, WorkoutRow};
use crate::error::AppError;

fn failed(what: &str, e: sqlx::Error) -> AppError {
    AppError::Internal(format!("Whoop : ecriture {what} impossible : {e}"))
}

pub async fn write_sleep(pool: &SqlitePool, rows: &[SleepRow]) -> Result<usize, AppError> {
    let mut tx = pool.begin().await?;
    for r in rows {
        sqlx::query(
            "INSERT INTO sleep_records (date, source, external_id, start_at, end_at, duration_total, \
             duration_deep, duration_light, duration_rem, duration_awake, total_timeinbed, \
             sleep_score, sleep_efficiency, rr_average) \
             VALUES (?1, 'whoop', ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13) \
             ON CONFLICT(date, source) DO UPDATE SET \
             external_id = excluded.external_id, start_at = excluded.start_at, \
             end_at = excluded.end_at, duration_total = excluded.duration_total, \
             duration_deep = excluded.duration_deep, duration_light = excluded.duration_light, \
             duration_rem = excluded.duration_rem, duration_awake = excluded.duration_awake, \
             total_timeinbed = excluded.total_timeinbed, sleep_score = excluded.sleep_score, \
             sleep_efficiency = excluded.sleep_efficiency, rr_average = excluded.rr_average",
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
        .bind(r.total_timeinbed)
        .bind(r.sleep_score)
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
            "INSERT INTO activity (date, source, external_id, calories_total, hr_average, hr_max) \
             VALUES (?1, 'whoop', ?2, ?3, ?4, ?5) \
             ON CONFLICT(date, source) DO UPDATE SET \
             external_id = excluded.external_id, calories_total = excluded.calories_total, \
             hr_average = excluded.hr_average, hr_max = excluded.hr_max",
        )
        .bind(&r.date)
        .bind(&r.external_id)
        .bind(r.calories_total)
        .bind(r.hr_average)
        .bind(r.hr_max)
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
             duration, distance, calories, hr_average, hr_max) \
             VALUES (?1, 'whoop', ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10) \
             ON CONFLICT(source, external_id) DO UPDATE SET \
             date = excluded.date, start_at = excluded.start_at, end_at = excluded.end_at, \
             category_label = excluded.category_label, duration = excluded.duration, \
             distance = excluded.distance, calories = excluded.calories, \
             hr_average = excluded.hr_average, hr_max = excluded.hr_max",
        )
        .bind(&r.date)
        .bind(&r.external_id)
        .bind(&r.start_at)
        .bind(&r.end_at)
        .bind(&r.category_label)
        .bind(r.duration)
        .bind(r.distance)
        .bind(r.calories)
        .bind(r.hr_average)
        .bind(r.hr_max)
        .execute(&mut *tx)
        .await
        .map_err(|e| failed("des entrainements", e))?;
    }
    tx.commit().await?;
    Ok(rows.len())
}
