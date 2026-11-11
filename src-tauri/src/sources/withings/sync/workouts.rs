use sqlx::SqlitePool;

use super::super::client::WithingsClient;
use super::super::mapping;
use super::super::types::*;
use super::{get_last_sync, update_last_sync};
use crate::error::AppError;

pub async fn sync_workouts(client: &WithingsClient, pool: &SqlitePool) -> Result<usize, AppError> {
    let last_sync = get_last_sync(pool, "workouts").await?;
    let cursor = chrono::Utc::now().timestamp();

    let (start_date, end_date) = super::compute_ymd_range(last_sync);

    let data_fields = "steps,distance,elevation,calories,hr_average,hr_min,hr_max,\
        hr_zone_0,hr_zone_1,hr_zone_2,hr_zone_3,spo2_average,intensity,\
        pause_duration,effortScore,pool_laps,pool_length";

    // Cluster A : collecter toutes les pages reseau AVANT la transaction
    // (centralise dans super::paginate_batch).
    let sd = start_date.as_str();
    let ed = end_date.as_str();
    let series = super::paginate_batch(|offset| {
        let offset_str = offset.to_string();
        async move {
            let mut params: Vec<(&str, &str)> = vec![
                ("action", "getworkouts"),
                ("startdateymd", sd),
                ("enddateymd", ed),
                ("data_fields", data_fields),
            ];
            if offset > 0 {
                params.push(("offset", &offset_str));
            }
            let resp: WithingsResponse<WorkoutBody> = client.post("/v2/measure", &params).await?;
            Ok((resp.body.series, resp.body.more))
        }
    })
    .await?;

    let mut count = 0usize;
    let mut tx = pool.begin().await?;
    for entry in &series {
        if persist_workout(&mut tx, entry).await? {
            count += 1;
        }
    }
    tx.commit().await?;

    update_last_sync(pool, "workouts", cursor).await?;
    Ok(count)
}

async fn persist_workout(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    entry: &WorkoutEntry,
) -> Result<bool, AppError> {
    let Some(date) = entry.date.clone() else {
        log::warn!("[sync] entrainement {:?} ignore : date absente", entry.id);
        return Ok(false);
    };

    let start_at = super::opt_ts_to_iso(entry.startdate);
    let end_at = super::opt_ts_to_iso(entry.enddate);

    let withings_id = entry.id.map(|v| v.to_string());

    let category = entry.category;
    let category_label = entry
        .category
        .map(|c| mapping::workout_category_label(c).to_string());

    let duration: Option<i64> = match (entry.startdate, entry.enddate) {
        (Some(s), Some(e)) => Some(e - s),
        _ => None,
    };

    sqlx::query(
        "INSERT INTO workouts (date, source, withings_id, start_at, end_at, category, category_label, duration, \
         steps, distance, elevation, calories, hr_average, hr_min, hr_max, \
         hr_zone_0, hr_zone_1, hr_zone_2, hr_zone_3, spo2_average, intensity, \
         pause_duration, effort_score, pool_laps, pool_length) \
         VALUES (?1, 'withings', ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, \
         ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24) \
         ON CONFLICT(date, withings_id) DO UPDATE SET \
         start_at = excluded.start_at, \
         end_at = excluded.end_at, \
         category = excluded.category, \
         category_label = excluded.category_label, \
         duration = excluded.duration, \
         steps = excluded.steps, \
         distance = excluded.distance, \
         elevation = excluded.elevation, \
         calories = excluded.calories, \
         hr_average = excluded.hr_average, \
         hr_min = excluded.hr_min, \
         hr_max = excluded.hr_max, \
         hr_zone_0 = excluded.hr_zone_0, \
         hr_zone_1 = excluded.hr_zone_1, \
         hr_zone_2 = excluded.hr_zone_2, \
         hr_zone_3 = excluded.hr_zone_3, \
         spo2_average = excluded.spo2_average, \
         intensity = excluded.intensity, \
         pause_duration = excluded.pause_duration, \
         effort_score = excluded.effort_score, \
         pool_laps = excluded.pool_laps, \
         pool_length = excluded.pool_length",
    )
    .bind(&date)
    .bind(&withings_id)
    .bind(&start_at)
    .bind(&end_at)
    .bind(category)
    .bind(&category_label)
    .bind(duration)
    .bind(entry.data.steps)
    .bind(entry.data.distance)
    .bind(entry.data.elevation)
    .bind(entry.data.calories)
    .bind(entry.data.hr_average)
    .bind(entry.data.hr_min)
    .bind(entry.data.hr_max)
    .bind(entry.data.hr_zone_0)
    .bind(entry.data.hr_zone_1)
    .bind(entry.data.hr_zone_2)
    .bind(entry.data.hr_zone_3)
    .bind(entry.data.spo2_average)
    .bind(entry.data.intensity)
    .bind(entry.data.pause_duration)
    .bind(entry.data.effort_score)
    .bind(entry.data.pool_laps)
    .bind(entry.data.pool_length)
    .execute(&mut **tx)
    .await
    .map_err(|e| AppError::Internal(format!("Workout upsert failed: {}", e)))?;

    Ok(true)
}
