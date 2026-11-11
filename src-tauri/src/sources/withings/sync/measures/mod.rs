mod body;
mod bp;
mod sync;
mod vitals;

pub use sync::sync_body_measures;

#[cfg(test)]
mod tests {
    use super::body::upsert_body_group;
    use super::bp::upsert_bp_group;
    use super::sync::{persist_group, MeasureCategory};
    use crate::db::bootstrap_schema;
    use crate::sources::withings::sync::metrics::preload_def_ids;
    use crate::sources::withings::types::{Measure, MeasureGroup};
    use sqlx::sqlite::SqlitePoolOptions;
    use sqlx::SqlitePool;

    async fn pool() -> SqlitePool {
        // max_connections(1) : garantit que la meme DB :memory: est vue par la tx
        // et les SELECT de verification.
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
    fn meas(measure_type: i64, value: i64, unit: i64) -> Measure {
        Measure {
            value,
            measure_type,
            unit,
        }
    }

    #[tokio::test]
    async fn body_group_multi_measures_writes_one_row() {
        let p = pool().await;
        let map = preload_def_ids(&p).await.unwrap();
        let mut tx = p.begin().await.unwrap();
        // weight 72.50 (type 1, 7250e-2), fat_ratio 18.00 (type 6, 1800e-2)
        let g = grp(1, vec![meas(1, 7250, -2), meas(6, 1800, -2)]);
        let n = upsert_body_group(&mut tx, &g, &map).await.unwrap();
        tx.commit().await.unwrap();
        assert_eq!(n, 1);
        let (w, fr): (f64, f64) = sqlx::query_as(
            "SELECT weight, fat_ratio FROM body_measures_legacy WHERE withings_grpid = '1'",
        )
        .fetch_one(&p)
        .await
        .unwrap();
        assert!((w - 72.5).abs() < 1e-6);
        assert!((fr - 18.0).abs() < 1e-6);
    }

    #[tokio::test]
    async fn body_group_readable_via_legacy_view() {
        // Regression vue read-only : l'ecriture cible body_measures_legacy et
        // reste lisible via la vue body_measures.
        let p = pool().await;
        let map = preload_def_ids(&p).await.unwrap();
        let mut tx = p.begin().await.unwrap();
        upsert_body_group(&mut tx, &grp(4, vec![meas(1, 8000, -2)]), &map)
            .await
            .unwrap();
        tx.commit().await.unwrap();
        let cnt: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM body_measures WHERE withings_grpid = '4'")
                .fetch_one(&p)
                .await
                .unwrap();
        assert_eq!(cnt, 1);
    }

    #[tokio::test]
    async fn bp_group_without_diastolic_writes_nothing() {
        let p = pool().await;
        let mut tx = p.begin().await.unwrap();
        // systolic seul (type 10), pas de diastolic (type 9)
        let n = upsert_bp_group(&mut tx, &grp(2, vec![meas(10, 120, 0)]))
            .await
            .unwrap();
        tx.commit().await.unwrap();
        assert_eq!(n, 0);
        let cnt: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM blood_pressure")
            .fetch_one(&p)
            .await
            .unwrap();
        assert_eq!(cnt, 0);
    }

    #[tokio::test]
    async fn bp_group_out_of_range_systolic_rejected() {
        let p = pool().await;
        let mut tx = p.begin().await.unwrap();
        // systolic 600 (>500) rejete -> pas de paire (sys,dia) -> Ok(0)
        let g = grp(3, vec![meas(10, 600, 0), meas(9, 80, 0)]);
        let n = upsert_bp_group(&mut tx, &g).await.unwrap();
        tx.commit().await.unwrap();
        assert_eq!(n, 0);
        let cnt: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM blood_pressure")
            .fetch_one(&p)
            .await
            .unwrap();
        assert_eq!(cnt, 0);
    }

    #[tokio::test]
    async fn persist_body_group_writes_the_legacy_table_only() {
        let p = pool().await;
        let map = preload_def_ids(&p).await.unwrap();
        let mut tx = p.begin().await.unwrap();
        persist_group(
            &mut tx,
            &grp(1, vec![meas(1, 7250, -2)]),
            MeasureCategory::Body,
            &map,
        )
        .await
        .unwrap();
        tx.commit().await.unwrap();
        let w: f64 = sqlx::query_scalar(
            "SELECT weight FROM body_measures_legacy WHERE withings_grpid = '1'",
        )
        .fetch_one(&p)
        .await
        .unwrap();
        assert!((w - 72.5).abs() < 1e-6);
        let mc: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM metrics m JOIN metric_definitions d ON d.id = m.definition_id WHERE d.slug = 'weight'",
        ).fetch_one(&p).await.unwrap();
        assert_eq!(mc, 0, "weight lives in body_measures_legacy only");
    }

    #[tokio::test]
    async fn body_group_out_of_range_weight_rejected_in_legacy() {
        let p = pool().await;
        let map = preload_def_ids(&p).await.unwrap();
        let mut tx = p.begin().await.unwrap();
        persist_group(
            &mut tx,
            &grp(99, vec![meas(1, 90000, -2)]),
            MeasureCategory::Body,
            &map,
        )
        .await
        .unwrap();
        tx.commit().await.unwrap();
        let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM body_measures_legacy")
            .fetch_one(&p)
            .await
            .unwrap();
        assert_eq!(n, 0);
    }

    #[tokio::test]
    async fn persist_vital_group_covers_pulse_wave_velocity() {
        let p = pool().await;
        let map = preload_def_ids(&p).await.unwrap();
        let mut tx = p.begin().await.unwrap();
        // type 91, 800e-2 = 8.0 m/s (bornes pwv [3,20])
        persist_group(
            &mut tx,
            &grp(2, vec![meas(91, 800, -2)]),
            MeasureCategory::Vital,
            &map,
        )
        .await
        .unwrap();
        tx.commit().await.unwrap();
        let vc: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM vitals_legacy WHERE vital_type = 'pulse_wave_velocity'",
        )
        .fetch_one(&p)
        .await
        .unwrap();
        assert_eq!(vc, 1, "91 -> vitals_legacy");
        let mc: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM metrics m JOIN metric_definitions d ON d.id = m.definition_id WHERE d.slug = 'pulse_wave_velocity'",
        ).fetch_one(&p).await.unwrap();
        assert_eq!(mc, 0, "91 lives in vitals_legacy only");
    }

    async fn heart_rate_metric_count(p: &SqlitePool) -> i64 {
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM metrics m JOIN metric_definitions d ON d.id = m.definition_id WHERE d.slug = 'heart_rate'",
        )
        .fetch_one(p)
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn a_pulse_that_belongs_to_a_blood_pressure_reading_stays_with_it() {
        let p = pool().await;
        let map = preload_def_ids(&p).await.unwrap();
        let mut tx = p.begin().await.unwrap();
        persist_group(
            &mut tx,
            &grp(5, vec![meas(10, 120, 0), meas(9, 80, 0), meas(11, 64, 0)]),
            MeasureCategory::BloodPressure,
            &map,
        )
        .await
        .unwrap();
        tx.commit().await.unwrap();
        let hr: i64 =
            sqlx::query_scalar("SELECT heart_rate FROM blood_pressure WHERE withings_grpid = '5'")
                .fetch_one(&p)
                .await
                .unwrap();
        assert_eq!(hr, 64);
        assert_eq!(heart_rate_metric_count(&p).await, 0);
        let others: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM metrics")
            .fetch_one(&p)
            .await
            .unwrap();
        assert_eq!(
            others, 0,
            "systolic and diastolic live in blood_pressure only"
        );
    }

    #[tokio::test]
    async fn a_pulse_without_blood_pressure_is_kept_in_metrics() {
        let p = pool().await;
        let map = preload_def_ids(&p).await.unwrap();
        let mut tx = p.begin().await.unwrap();
        persist_group(
            &mut tx,
            &grp(6, vec![meas(11, 71, 0)]),
            MeasureCategory::BloodPressure,
            &map,
        )
        .await
        .unwrap();
        tx.commit().await.unwrap();
        let bp: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM blood_pressure")
            .fetch_one(&p)
            .await
            .unwrap();
        assert_eq!(bp, 0);
        assert_eq!(heart_rate_metric_count(&p).await, 1);
    }

    #[tokio::test]
    async fn an_out_of_range_pulse_alone_is_rejected() {
        let p = pool().await;
        let map = preload_def_ids(&p).await.unwrap();
        let mut tx = p.begin().await.unwrap();
        persist_group(
            &mut tx,
            &grp(7, vec![meas(11, 900, 0)]),
            MeasureCategory::BloodPressure,
            &map,
        )
        .await
        .unwrap();
        tx.commit().await.unwrap();
        assert_eq!(heart_rate_metric_count(&p).await, 0);
    }
}
