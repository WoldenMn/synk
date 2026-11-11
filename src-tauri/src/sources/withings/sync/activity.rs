use sqlx::SqlitePool;

use super::super::client::WithingsClient;
use super::super::types::*;
use super::{get_last_sync, update_last_sync};
use crate::error::AppError;

pub async fn sync_activity(client: &WithingsClient, pool: &SqlitePool) -> Result<usize, AppError> {
    let last_sync = get_last_sync(pool, "activity").await?;
    let cursor = chrono::Utc::now().timestamp();

    let (start_date, end_date) = super::compute_ymd_range(last_sync);

    let data_fields = "steps,distance,elevation,soft,moderate,intense,active,calories,totalcalories,hr_average,hr_min,hr_max,hr_zone_0,hr_zone_1,hr_zone_2,hr_zone_3";

    // Cluster A : collecter toutes les pages reseau AVANT la transaction
    // (centralise dans super::paginate_batch).
    let sd = start_date.as_str();
    let ed = end_date.as_str();
    let activities = super::paginate_batch(|offset| {
        let offset_str = offset.to_string();
        async move {
            let mut params: Vec<(&str, &str)> = vec![
                ("action", "getactivity"),
                ("startdateymd", sd),
                ("enddateymd", ed),
                ("data_fields", data_fields),
            ];
            if offset > 0 {
                params.push(("offset", &offset_str));
            }
            let resp: WithingsResponse<ActivityBody> = client.post("/v2/measure", &params).await?;
            Ok((resp.body.activities, resp.body.more))
        }
    })
    .await?;

    let mut count = 0usize;
    let mut tx = pool.begin().await?;
    for entry in &activities {
        sqlx::query(
            "INSERT INTO activity (date, source, steps, distance, elevation, calories_active, calories_total, \
             duration_soft, duration_moderate, duration_intense, duration_active, hr_average, hr_min, hr_max, \
             hr_zone_0, hr_zone_1, hr_zone_2, hr_zone_3) \
             VALUES (?1, 'withings', ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17) \
             ON CONFLICT(date, source) DO UPDATE SET \
             steps = excluded.steps, \
             distance = excluded.distance, \
             elevation = excluded.elevation, \
             calories_active = excluded.calories_active, \
             calories_total = excluded.calories_total, \
             duration_soft = excluded.duration_soft, \
             duration_moderate = excluded.duration_moderate, \
             duration_intense = excluded.duration_intense, \
             duration_active = excluded.duration_active, \
             hr_average = excluded.hr_average, \
             hr_min = excluded.hr_min, \
             hr_max = excluded.hr_max, \
             hr_zone_0 = excluded.hr_zone_0, \
             hr_zone_1 = excluded.hr_zone_1, \
             hr_zone_2 = excluded.hr_zone_2, \
             hr_zone_3 = excluded.hr_zone_3",
        )
        .bind(&entry.date)
        .bind(entry.steps)
        .bind(entry.distance)
        .bind(entry.elevation)
        .bind(entry.calories)
        .bind(entry.totalcalories)
        .bind(entry.soft)
        .bind(entry.moderate)
        .bind(entry.intense)
        .bind(entry.active)
        .bind(entry.hr_average)
        .bind(entry.hr_min)
        .bind(entry.hr_max)
        .bind(entry.hr_zone_0)
        .bind(entry.hr_zone_1)
        .bind(entry.hr_zone_2)
        .bind(entry.hr_zone_3)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Internal(format!("Activity upsert failed: {}", e)))?;

        count += 1;
    }
    tx.commit().await?;

    update_last_sync(pool, "activity", cursor).await?;
    Ok(count)
}
