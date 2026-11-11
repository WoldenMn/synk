use sqlx::SqlitePool;

use super::super::client::WithingsClient;
use super::super::types::*;
use super::{get_last_sync, update_last_sync};
use crate::error::AppError;

pub async fn sync_heart(client: &WithingsClient, pool: &SqlitePool) -> Result<usize, AppError> {
    let last_sync = get_last_sync(pool, "heart").await?;

    let start_ts = if last_sync == 0 {
        chrono::Utc::now()
            .checked_sub_signed(chrono::Duration::days(180))
            .unwrap_or_else(chrono::Utc::now)
            .timestamp()
    } else {
        last_sync
    };
    let end_ts = chrono::Utc::now().timestamp();

    let start_str = start_ts.to_string();
    let end_str = end_ts.to_string();

    // Cluster A (#1) : collecter toutes les pages reseau AVANT la transaction
    // (centralise dans super::paginate_batch).
    let ss = start_str.as_str();
    let es = end_str.as_str();
    let series = super::paginate_batch(|offset| {
        let offset_str = offset.to_string();
        async move {
            let mut params: Vec<(&str, &str)> =
                vec![("action", "list"), ("startdate", ss), ("enddate", es)];
            if offset > 0 {
                params.push(("offset", &offset_str));
            }
            let resp: WithingsResponse<HeartListBody> = client.post("/v2/heart", &params).await?;
            Ok((resp.body.series, resp.body.more))
        }
    })
    .await?;

    let mut count = 0usize;
    let mut tx = pool.begin().await?;
    for entry in &series {
        // B-13 : timestamp est la clé UNIQUE(timestamp, source). Un timestamp hors plage
        // produirait une chaîne vide invalide en DB — on skippe et on loggue.
        let timestamp = match super::ts_to_iso(entry.timestamp) {
            Some(ts) => ts,
            None => {
                log::warn!("Heart: timestamp hors plage ignoré: {}", entry.timestamp);
                continue;
            }
        };

        let ecg_status = entry.ecg.as_ref().and_then(|e| e.afib);
        let heart_rate = entry.heart_rate.as_ref().and_then(|h| h.value);
        let qrs_duration = entry.heart_rate.as_ref().and_then(|h| h.qrs_duration);
        let withings_ecg_id = entry
            .ecg
            .as_ref()
            .and_then(|e| e.signalid)
            .map(|s| s.to_string());

        sqlx::query(
            "INSERT INTO heart_records (timestamp, source, withings_ecg_id, ecg_status, heart_rate, qrs_duration) \
             VALUES (?1, 'withings', ?2, ?3, ?4, ?5) \
             ON CONFLICT(timestamp, source) DO UPDATE SET \
             withings_ecg_id = excluded.withings_ecg_id, \
             ecg_status = excluded.ecg_status, \
             heart_rate = excluded.heart_rate, \
             qrs_duration = excluded.qrs_duration",
        )
        .bind(&timestamp)
        .bind(&withings_ecg_id)
        .bind(ecg_status)
        .bind(heart_rate)
        .bind(qrs_duration)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Internal(format!("Heart upsert failed: {}", e)))?;

        count += 1;
    }
    tx.commit().await?;

    update_last_sync(pool, "heart", end_ts).await?;
    Ok(count)
}
