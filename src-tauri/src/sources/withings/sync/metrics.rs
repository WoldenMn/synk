use sqlx::{SqliteConnection, SqlitePool};
use std::collections::HashMap;

use super::super::mapping;
use super::super::types::*;
use crate::error::AppError;

/// Precharge le mapping `(withings_measure_type -> def_id)` pour TOUS les
/// measure_types presents dans `metric_definitions`. Evite le N+1 dans la
/// boucle d.ecriture (V2.7.0 faisait un SELECT id par measure).
///
/// HashMap inclut uniquement les definitions avec `withings_type IS NOT NULL`
/// (entries body_composition, body_scan, vital_bp). Slugs custom/derived
/// n'ont pas de withings_type donc n'apparaissent pas — coherent avec le
/// synchro, qui ne traite que des measures Withings.
/// Métadonnées d'une définition métrique pertinentes pour l'écriture (validation bornes DB-002).
#[derive(Clone, Copy)]
pub(super) struct DefMeta {
    pub(super) id: i64,
    pub(super) min: Option<f64>,
    pub(super) max: Option<f64>,
}

pub(super) async fn preload_def_ids(pool: &SqlitePool) -> Result<HashMap<i64, DefMeta>, AppError> {
    let rows: Vec<(i64, i64, Option<f64>, Option<f64>)> = sqlx::query_as(
        "SELECT withings_type, id, min_value, max_value FROM metric_definitions WHERE withings_type IS NOT NULL"
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(wt, id, min, max)| (wt, DefMeta { id, min, max }))
        .collect())
}

pub(super) async fn upsert_pulse_alone(
    conn: &mut SqliteConnection,
    grp: &MeasureGroup,
    def_id_map: &HashMap<i64, DefMeta>,
) -> Result<usize, AppError> {
    let Some(measured_at) = super::measured_at_iso(grp.date) else {
        return Ok(0);
    };
    let Some(meta) = def_id_map.get(&mapping::MEASURE_HEART_RATE).copied() else {
        log::warn!("heart_rate absent de metric_definitions, pouls ignore");
        return Ok(0);
    };
    let grpid = grp.grpid.to_string();
    let mut count = 0usize;

    for m in grp
        .measures
        .iter()
        .filter(|m| m.measure_type == mapping::MEASURE_HEART_RATE)
    {
        let raw = mapping::convert_measure(m.value, m.unit);
        let Some(value) = mapping::sanitize_measure(raw, meta.min, meta.max) else {
            log::warn!(
                "pouls rejete (non-fini ou hors [{:?},{:?}]): {}",
                meta.min,
                meta.max,
                raw
            );
            continue;
        };
        sqlx::query(
            "INSERT INTO metrics (definition_id, measured_at, value, source, external_id) \
             VALUES (?1, ?2, ?3, 'withings', ?4) \
             ON CONFLICT(definition_id, measured_at, source, external_id) \
             DO UPDATE SET value = excluded.value",
        )
        .bind(meta.id)
        .bind(&measured_at)
        .bind(value)
        .bind(&grpid)
        .execute(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(format!("metrics upsert failed: {e}")))?;
        count += 1;
    }

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::bootstrap_schema;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn pool() -> SqlitePool {
        let p = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(":memory:")
            .await
            .unwrap();
        bootstrap_schema(&p).await.unwrap();
        p
    }

    fn grp(grpid: i64, measures: Vec<Measure>) -> MeasureGroup {
        MeasureGroup {
            grpid,
            date: 1_700_000_000,
            measures,
        }
    }

    fn pulse(value: i64) -> Measure {
        Measure {
            value,
            measure_type: 11,
            unit: 0,
        }
    }

    async fn metric_count(p: &SqlitePool) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM metrics")
            .fetch_one(p)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn a_pulse_within_bounds_is_written() {
        let p = pool().await;
        let map = preload_def_ids(&p).await.unwrap();
        let mut tx = p.begin().await.unwrap();
        let n = upsert_pulse_alone(&mut tx, &grp(10, vec![pulse(64)]), &map)
            .await
            .unwrap();
        tx.commit().await.unwrap();
        assert_eq!(n, 1);
        assert_eq!(metric_count(&p).await, 1);
    }

    #[tokio::test]
    async fn a_pulse_out_of_bounds_is_rejected() {
        let p = pool().await;
        let map = preload_def_ids(&p).await.unwrap();
        let mut tx = p.begin().await.unwrap();
        let n = upsert_pulse_alone(&mut tx, &grp(11, vec![pulse(900)]), &map)
            .await
            .unwrap();
        tx.commit().await.unwrap();
        assert_eq!(n, 0);
        assert_eq!(metric_count(&p).await, 0);
    }

    #[tokio::test]
    async fn only_the_pulse_of_a_group_is_written() {
        let p = pool().await;
        let map = preload_def_ids(&p).await.unwrap();
        let mut tx = p.begin().await.unwrap();
        let other = Measure {
            value: 7250,
            measure_type: 1,
            unit: -2,
        };
        let n = upsert_pulse_alone(&mut tx, &grp(12, vec![other, pulse(70)]), &map)
            .await
            .unwrap();
        tx.commit().await.unwrap();
        assert_eq!(n, 1);
        assert_eq!(metric_count(&p).await, 1);
    }
}
