use crate::error::AppError;
use std::collections::HashMap;

use sqlx::SqliteConnection;

use crate::sources::withings::mapping;
use crate::sources::withings::types::MeasureGroup;

use super::super::metrics::DefMeta;

pub(super) async fn upsert_body_group(
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

    let mut weight: Option<f64> = None;
    let mut fat_ratio: Option<f64> = None;
    let mut fat_mass: Option<f64> = None;
    let mut fat_free_mass: Option<f64> = None;
    let mut muscle_mass: Option<f64> = None;
    let mut bone_mass: Option<f64> = None;
    let mut hydration: Option<f64> = None;

    let mut has_body = false;
    for m in &grp.measures {
        if mapping::is_body_composition(m.measure_type) {
            let raw = mapping::convert_measure(m.value, m.unit);
            let (min, max) = def_id_map
                .get(&m.measure_type)
                .map(|d| (d.min, d.max))
                .unwrap_or((None, None));
            let Some(val) = mapping::sanitize_measure(raw, min, max) else {
                log::warn!(
                    "body measure rejected (non-finite/out-of-range): {} (type={})",
                    raw,
                    m.measure_type
                );
                continue;
            };
            has_body = true;
            match m.measure_type {
                mapping::MEASURE_WEIGHT => weight = Some(val),
                mapping::MEASURE_FAT_RATIO => fat_ratio = Some(val),
                mapping::MEASURE_FAT_MASS => fat_mass = Some(val),
                mapping::MEASURE_FAT_FREE_MASS => fat_free_mass = Some(val),
                mapping::MEASURE_MUSCLE_MASS => muscle_mass = Some(val),
                mapping::MEASURE_BONE_MASS => bone_mass = Some(val),
                mapping::MEASURE_HYDRATION => hydration = Some(val),
                _ => {}
            }
        }
    }

    if !has_body {
        return Ok(0);
    }

    sqlx::query(
        "INSERT INTO body_measures_legacy (measured_at, source, withings_grpid, weight, fat_ratio, fat_mass, fat_free_mass, muscle_mass, bone_mass, hydration) \
         VALUES (?1, 'withings', ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9) \
         ON CONFLICT(withings_grpid) DO UPDATE SET \
         weight = excluded.weight, \
         fat_ratio = excluded.fat_ratio, \
         fat_mass = excluded.fat_mass, \
         fat_free_mass = excluded.fat_free_mass, \
         muscle_mass = excluded.muscle_mass, \
         bone_mass = excluded.bone_mass, \
         hydration = excluded.hydration",
    )
    .bind(&measured_at)
    .bind(&grpid)
    .bind(weight)
    .bind(fat_ratio)
    .bind(fat_mass)
    .bind(fat_free_mass)
    .bind(muscle_mass)
    .bind(bone_mass)
    .bind(hydration)
    .execute(&mut *conn)
    .await
    .map_err(|e| AppError::Internal(format!("Body upsert failed: {}", e)))?;

    Ok(1)
}
