use crate::error::AppError;
use sqlx::SqliteConnection;

use crate::sources::withings::mapping;
use crate::sources::withings::types::MeasureGroup;

pub(super) async fn upsert_bp_group(
    conn: &mut SqliteConnection,
    grp: &MeasureGroup,
) -> Result<usize, AppError> {
    let Some(measured_at) = super::super::measured_at_iso(grp.date) else {
        log::warn!(
            "[sync] groupe {} ignore : horodatage hors plage ({})",
            grp.grpid,
            grp.date
        );
        return Ok(0);
    };
    let grpid = grp.grpid.to_string();

    let mut systolic: Option<i64> = None;
    let mut diastolic: Option<i64> = None;
    let mut heart_rate: Option<i64> = None;

    let mut has_bp = false;
    for m in &grp.measures {
        if mapping::is_blood_pressure(m.measure_type) {
            has_bp = true;
            let raw = mapping::convert_measure(m.value, m.unit);
            // Clamp explicite avant cast f64 -> i64 (NaN, +/-Infinity, hors-borne -> ignore la mesure).
            // Bornes physio realistes : 0-500 (mmHg systolique max ~250, BPM max ~250).
            if !raw.is_finite() || !(0.0..=500.0).contains(&raw) {
                log::warn!(
                    "BP measure out of range or non-finite: {} (type={})",
                    raw,
                    m.measure_type
                );
                continue;
            }
            let val = raw.round() as i64;
            match m.measure_type {
                mapping::MEASURE_SYSTOLIC => systolic = Some(val),
                mapping::MEASURE_DIASTOLIC => diastolic = Some(val),
                mapping::MEASURE_HEART_RATE => heart_rate = Some(val),
                _ => {}
            }
        }
    }

    if !has_bp {
        return Ok(0);
    }

    let (Some(sys), Some(dia)) = (systolic, diastolic) else {
        return Ok(0);
    };

    sqlx::query(
        "INSERT INTO blood_pressure (measured_at, source, withings_grpid, systolic, diastolic, heart_rate) \
         VALUES (?1, 'withings', ?2, ?3, ?4, ?5) \
         ON CONFLICT(withings_grpid) DO UPDATE SET \
         systolic = excluded.systolic, \
         diastolic = excluded.diastolic, \
         heart_rate = excluded.heart_rate",
    )
    .bind(&measured_at)
    .bind(&grpid)
    .bind(sys)
    .bind(dia)
    .bind(heart_rate)
    .execute(&mut *conn)
    .await
    .map_err(|e| AppError::Internal(format!("BP upsert failed: {}", e)))?;

    Ok(1)
}
