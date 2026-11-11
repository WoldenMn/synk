use crate::error::AppError;
use std::collections::HashMap;

use sqlx::SqliteConnection;

use crate::sources::withings::mapping;
use crate::sources::withings::types::MeasureGroup;

use super::super::metrics::DefMeta;

pub(super) async fn upsert_vital_group(
    conn: &mut SqliteConnection,
    grp: &MeasureGroup,
    def_id_map: &HashMap<i64, DefMeta>,
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
    let mut count = 0usize;

    for m in &grp.measures {
        if let Some(vital_type) = mapping::vital_type_name(m.measure_type) {
            let raw = mapping::convert_measure(m.value, m.unit);
            let (min, max) = def_id_map
                .get(&m.measure_type)
                .map(|d| (d.min, d.max))
                .unwrap_or((None, None));
            let Some(val) = mapping::sanitize_measure(raw, min, max) else {
                log::warn!(
                    "vital measure rejected (non-finite/out-of-range): {} (type={})",
                    raw,
                    m.measure_type
                );
                continue;
            };

            sqlx::query(
                "INSERT INTO vitals_legacy (measured_at, source, withings_grpid, vital_type, value) \
                 VALUES (?1, 'withings', ?2, ?3, ?4) \
                 ON CONFLICT(withings_grpid, vital_type) DO UPDATE SET \
                 value = excluded.value",
            )
            .bind(&measured_at)
            .bind(&grpid)
            .bind(vital_type)
            .bind(val)
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("Vital upsert failed: {}", e)))?;

            count += 1;
        }
    }

    Ok(count)
}
