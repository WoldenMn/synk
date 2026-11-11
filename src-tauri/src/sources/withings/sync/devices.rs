use sqlx::SqlitePool;

use super::super::client::WithingsClient;
use super::super::mapping;
use super::super::types::*;
use super::update_last_sync;
use crate::error::AppError;

pub async fn sync_devices(client: &WithingsClient, pool: &SqlitePool) -> Result<usize, AppError> {
    let cursor = chrono::Utc::now().timestamp();
    let params: Vec<(&str, &str)> = vec![("action", "getdevice")];

    let resp: WithingsResponse<DeviceBody> = client.post("/v2/user", &params).await?;

    let mut count = 0usize;

    let mut tx = pool.begin().await?;

    for entry in &resp.body.devices {
        let Some(device_id) = entry.deviceid.clone() else {
            log::warn!(
                "[sync] appareil {:?} ignore : identifiant absent",
                entry.model
            );
            continue;
        };
        let device_type = &entry.device_type;
        let model = entry
            .model_id
            .and_then(mapping::friendly_device_name)
            .or(entry.model.clone());
        let model_id = entry.model_id;
        let battery = &entry.battery;

        let last_session_date = super::opt_ts_to_iso(entry.last_session_date);
        let first_session_date = super::opt_ts_to_iso(entry.first_session_date);

        sqlx::query(
            "INSERT INTO withings_devices (device_id, device_type, model, model_id, battery, last_session_date, first_session_date) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7) \
             ON CONFLICT(device_id) DO UPDATE SET \
             device_type = excluded.device_type, \
             model = excluded.model, \
             model_id = excluded.model_id, \
             battery = excluded.battery, \
             last_session_date = excluded.last_session_date, \
             first_session_date = excluded.first_session_date",
        )
        .bind(&device_id)
        .bind(device_type)
        .bind(model)
        .bind(model_id)
        .bind(battery)
        .bind(&last_session_date)
        .bind(&first_session_date)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Internal(format!("Device upsert failed: {}", e)))?;

        count += 1;
    }

    tx.commit().await?;

    update_last_sync(pool, "devices", cursor).await?;
    Ok(count)
}
