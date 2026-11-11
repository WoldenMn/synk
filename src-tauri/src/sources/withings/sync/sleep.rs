use sqlx::SqlitePool;

use super::super::client::WithingsClient;
use super::super::types::*;
use super::{get_last_sync, update_last_sync};
use crate::error::AppError;

pub async fn sync_sleep(client: &WithingsClient, pool: &SqlitePool) -> Result<usize, AppError> {
    let last_sync = get_last_sync(pool, "sleep").await?;
    let cursor = chrono::Utc::now().timestamp();

    let (start_date, end_date) = super::compute_ymd_range(last_sync);

    let data_fields = "total_sleep_time,deepsleepduration,lightsleepduration,remsleepduration,\
        wakeupduration,durationtosleep,durationtowakeup,sleep_score,\
        hr_average,hr_min,hr_max,rr_average,snoring,snoringepisodecount,\
        sleep_efficiency,nb_rem_episodes,out_of_bed_count,waso,total_timeinbed,\
        apnea_hypopnea_index,breathing_disturbances_intensity,asleepduration,night_events";

    // Cluster A (#9) : collecter toutes les pages reseau AVANT la transaction
    // (centralise dans super::paginate_batch).
    let sd = start_date.as_str();
    let ed = end_date.as_str();
    let series = super::paginate_batch(|offset| {
        let offset_str = offset.to_string();
        async move {
            let mut params: Vec<(&str, &str)> = vec![
                ("action", "getsummary"),
                ("startdateymd", sd),
                ("enddateymd", ed),
                ("data_fields", data_fields),
            ];
            if offset > 0 {
                params.push(("offset", &offset_str));
            }
            let resp: WithingsResponse<SleepBody> = client.post("/v2/sleep", &params).await?;
            Ok((resp.body.series, resp.body.more))
        }
    })
    .await?;

    for (date, n) in same_day_series(&series) {
        log::warn!(
            "[sync] sommeil : {n} series pour le {date}, une seule est gardee (cle date et source)"
        );
    }

    let mut count = 0usize;
    let mut tx = pool.begin().await?;
    for entry in &series {
        persist_sleep(&mut tx, entry).await?;
        count += 1;
    }
    tx.commit().await?;

    update_last_sync(pool, "sleep", cursor).await?;
    Ok(count)
}

fn same_day_series(series: &[SleepEntry]) -> Vec<(String, usize)> {
    let mut per_date: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for entry in series {
        *per_date.entry(entry.date.as_str()).or_default() += 1;
    }
    per_date
        .into_iter()
        .filter(|(_, n)| *n > 1)
        .map(|(date, n)| (date.to_string(), n))
        .collect()
}

async fn persist_sleep(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    entry: &SleepEntry,
) -> Result<(), AppError> {
    let start_at = super::opt_ts_to_iso(entry.startdate);
    let end_at = super::opt_ts_to_iso(entry.enddate);

    sqlx::query(
        "INSERT INTO sleep_records (date, source, start_at, end_at, duration_total, duration_deep, duration_light, \
         duration_rem, duration_awake, duration_to_sleep, duration_to_wakeup, sleep_score, \
         hr_average, hr_min, hr_max, rr_average, snoring, snoring_count, \
         sleep_efficiency, nb_rem_episodes, out_of_bed_count, waso, total_timeinbed, \
         apnea_hypopnea_index, breathing_disturbances_intensity, asleepduration, night_events) \
         VALUES (?1, 'withings', ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, \
         ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26) \
         ON CONFLICT(date, source) DO UPDATE SET \
         start_at = excluded.start_at, \
         end_at = excluded.end_at, \
         duration_total = excluded.duration_total, \
         duration_deep = excluded.duration_deep, \
         duration_light = excluded.duration_light, \
         duration_rem = excluded.duration_rem, \
         duration_awake = excluded.duration_awake, \
         duration_to_sleep = excluded.duration_to_sleep, \
         duration_to_wakeup = excluded.duration_to_wakeup, \
         sleep_score = excluded.sleep_score, \
         hr_average = excluded.hr_average, \
         hr_min = excluded.hr_min, \
         hr_max = excluded.hr_max, \
         rr_average = excluded.rr_average, \
         snoring = excluded.snoring, \
         snoring_count = excluded.snoring_count, \
         sleep_efficiency = excluded.sleep_efficiency, \
         nb_rem_episodes = excluded.nb_rem_episodes, \
         out_of_bed_count = excluded.out_of_bed_count, \
         waso = excluded.waso, \
         total_timeinbed = excluded.total_timeinbed, \
         apnea_hypopnea_index = excluded.apnea_hypopnea_index, \
         breathing_disturbances_intensity = excluded.breathing_disturbances_intensity, \
         asleepduration = excluded.asleepduration, \
         night_events = excluded.night_events",
    )
    .bind(&entry.date)
    .bind(&start_at)
    .bind(&end_at)
    .bind(entry.data.total_sleep_time)
    .bind(entry.data.deepsleepduration)
    .bind(entry.data.lightsleepduration)
    .bind(entry.data.remsleepduration)
    .bind(entry.data.wakeupduration)
    .bind(entry.data.durationtosleep)
    .bind(entry.data.durationtowakeup)
    .bind(entry.data.sleep_score)
    .bind(entry.data.hr_average)
    .bind(entry.data.hr_min)
    .bind(entry.data.hr_max)
    .bind(entry.data.rr_average)
    .bind(entry.data.snoring)
    .bind(entry.data.snoringepisodecount)
    .bind(entry.data.sleep_efficiency)
    .bind(entry.data.nb_rem_episodes)
    .bind(entry.data.out_of_bed_count)
    .bind(entry.data.waso)
    .bind(entry.data.total_timeinbed)
    .bind(entry.data.apnea_hypopnea_index)
    .bind(entry.data.breathing_disturbances_intensity)
    .bind(entry.data.asleepduration)
    .bind(entry.data.night_events)
    .execute(&mut **tx)
    .await
    .map_err(|e| AppError::Internal(format!("Sleep upsert failed: {}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(date: &str, start: i64) -> SleepEntry {
        serde_json::from_value(serde_json::json!({
            "date": date,
            "startdate": start,
            "enddate": start + 3600,
            "data": {}
        }))
        .expect("serie")
    }

    #[test]
    fn two_series_on_one_date_are_reported() {
        let series = vec![
            entry("2026-10-24", 1),
            entry("2026-10-25", 2),
            entry("2026-10-25", 3),
            entry("2026-10-25", 4),
        ];
        assert_eq!(
            same_day_series(&series),
            vec![("2026-10-25".to_string(), 3)]
        );
    }

    #[test]
    fn one_series_per_date_reports_nothing() {
        let series = vec![entry("2026-10-24", 1), entry("2026-10-25", 2)];
        assert!(same_day_series(&series).is_empty());
        assert!(same_day_series(&[]).is_empty());
    }
}
