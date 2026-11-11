use sqlx::SqlitePool;
use synk_lib::metric_repository::{
    read_metric_latest, read_metric_series, source_of, LegacyTable, MetricSource,
};

async fn def_id(pool: &SqlitePool, slug: &str) -> i64 {
    sqlx::query_scalar("SELECT id FROM metric_definitions WHERE slug = ?1")
        .bind(slug)
        .fetch_one(pool)
        .await
        .expect("definition")
}

async fn put_metric(pool: &SqlitePool, slug: &str, at: &str, value: f64, ext: &str) {
    sqlx::query(
        "INSERT INTO metrics (definition_id, measured_at, value, source, external_id) \
         VALUES (?1, ?2, ?3, 'withings', ?4)",
    )
    .bind(def_id(pool, slug).await)
    .bind(at)
    .bind(value)
    .bind(ext)
    .execute(pool)
    .await
    .expect("metrique");
}

async fn put_bp(pool: &SqlitePool, at: &str, grp: &str, heart_rate: Option<i64>) {
    sqlx::query(
        "INSERT INTO blood_pressure (measured_at, source, withings_grpid, systolic, diastolic, heart_rate) \
         VALUES (?1, 'withings', ?2, 120, 80, ?3)",
    )
    .bind(at)
    .bind(grp)
    .bind(heart_rate)
    .execute(pool)
    .await
    .expect("tension");
}

#[tokio::test]
async fn every_metric_has_exactly_one_source() {
    let pool = crate::common::memory_pool().await;
    let slugs: Vec<String> = sqlx::query_scalar("SELECT slug FROM metric_definitions")
        .fetch_all(&pool)
        .await
        .expect("definitions");
    assert!(slugs.len() >= 42, "{} definitions", slugs.len());
    for slug in slugs {
        let source = source_of(&pool, &slug).await.expect("lecture");
        assert!(source.is_some(), "{slug} : aucune source");
    }
}

#[tokio::test]
async fn the_source_follows_the_inventory() {
    let pool = crate::common::memory_pool().await;
    let of = |slug: &'static str| {
        let pool = pool.clone();
        async move { source_of(&pool, slug).await.unwrap().unwrap().1 }
    };
    assert_eq!(
        of("weight").await,
        MetricSource::Legacy(LegacyTable::BodyMeasures)
    );
    assert_eq!(
        of("pulse_wave_velocity").await,
        MetricSource::Legacy(LegacyTable::Vitals)
    );
    assert_eq!(
        of("temperature").await,
        MetricSource::Legacy(LegacyTable::Vitals)
    );
    assert_eq!(
        of("waist").await,
        MetricSource::Legacy(LegacyTable::ManualMeasurements)
    );
    assert_eq!(
        of("systolic").await,
        MetricSource::Legacy(LegacyTable::BloodPressure)
    );
    assert_eq!(
        of("sleep_score").await,
        MetricSource::Legacy(LegacyTable::SleepRecords)
    );
    assert_eq!(
        of("steps").await,
        MetricSource::Legacy(LegacyTable::Activity)
    );
    assert_eq!(
        of("heart_rate").await,
        MetricSource::Split(LegacyTable::BloodPressure)
    );
    assert_eq!(of("ffmi").await, MetricSource::Metrics);
    assert!(source_of(&pool, "nonexistent_slug")
        .await
        .unwrap()
        .is_none());
}

#[tokio::test]
async fn a_legacy_metric_ignores_its_leftover_rows_in_metrics() {
    let pool = crate::common::memory_pool().await;
    put_metric(&pool, "weight", "2026-06-01T08:00:00Z", 70.0, "g1").await;
    put_metric(&pool, "weight", "2026-06-02T08:00:00Z", 99.0, "g2").await;
    sqlx::query(
        "INSERT INTO body_measures_legacy (measured_at, source, withings_grpid, weight) \
         VALUES ('2026-06-01T08:00:00Z', 'withings', 'g1', 70.0)",
    )
    .execute(&pool)
    .await
    .unwrap();

    let points = read_metric_series(&pool, "weight", "2026-01-01", "2026-12-31")
        .await
        .unwrap();

    assert_eq!(points.len(), 1, "une seule source : la table historique");
    assert_eq!(points[0].value, 70.0);
    let latest = read_metric_latest(&pool, "weight").await.unwrap().unwrap();
    assert_eq!(latest.value, 70.0, "le reste de metrics n'est pas lu");
}

#[tokio::test]
async fn two_legacy_rows_at_one_instant_make_one_point() {
    let pool = crate::common::memory_pool().await;
    for (src, grp) in [("withings", "g1"), ("manual", "g2")] {
        sqlx::query(
            "INSERT INTO body_measures_legacy (measured_at, source, withings_grpid, weight) \
             VALUES ('2026-06-01T08:00:00Z', ?1, ?2, 70.0)",
        )
        .bind(src)
        .bind(grp)
        .execute(&pool)
        .await
        .unwrap();
    }

    let points =
        synk_lib::services::metrics::select_series(&pool, "weight", "2026-01-01", "2026-12-31")
            .await
            .unwrap();

    assert_eq!(points.len(), 1, "un slug, une valeur par instant");
}

#[tokio::test]
async fn a_metric_without_a_table_is_read_from_metrics() {
    let pool = crate::common::memory_pool().await;
    put_metric(&pool, "ffmi", "2026-06-01T08:00:00Z", 19.5, "a").await;
    put_metric(&pool, "ffmi", "2026-06-03T08:00:00Z", 20.1, "b").await;

    let points = read_metric_series(&pool, "ffmi", "2026-06-01", "2026-06-30")
        .await
        .unwrap();
    let latest = read_metric_latest(&pool, "ffmi").await.unwrap().unwrap();

    assert_eq!(points.len(), 2);
    assert_eq!(latest.value, 20.1);
}

#[tokio::test]
async fn heart_rate_keeps_both_kinds_of_pulse_and_counts_a_shared_one_once() {
    let pool = crate::common::memory_pool().await;
    put_bp(&pool, "2026-06-01T08:00:00Z", "b1", Some(60)).await;
    put_metric(&pool, "heart_rate", "2026-06-01T08:00:00Z", 60.0, "b1").await;
    put_bp(&pool, "2026-06-02T08:00:00Z", "b2", Some(65)).await;
    put_metric(&pool, "heart_rate", "2026-06-03T08:00:00Z", 70.0, "alone").await;
    put_bp(&pool, "2026-06-04T08:00:00Z", "b3", None).await;

    let points = read_metric_series(&pool, "heart_rate", "2026-06-01", "2026-06-30")
        .await
        .unwrap();

    let seen: Vec<(&str, f64)> = points
        .iter()
        .map(|p| (p.measured_at.as_str(), p.value))
        .collect();
    assert_eq!(
        seen,
        vec![
            ("2026-06-01T08:00:00Z", 60.0),
            ("2026-06-02T08:00:00Z", 65.0),
            ("2026-06-03T08:00:00Z", 70.0),
        ]
    );
}

#[tokio::test]
async fn the_latest_pulse_is_the_most_recent_of_both_sources() {
    let pool = crate::common::memory_pool().await;
    put_bp(&pool, "2026-06-02T08:00:00Z", "b2", Some(65)).await;
    put_metric(&pool, "heart_rate", "2026-06-03T08:00:00Z", 70.0, "alone").await;
    let latest = read_metric_latest(&pool, "heart_rate")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(latest.value, 70.0);

    put_bp(&pool, "2026-06-05T08:00:00Z", "b5", Some(58)).await;
    let latest = read_metric_latest(&pool, "heart_rate")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(latest.value, 58.0);
}

#[test]
fn a_body_composition_slug_without_a_column_has_no_source() {
    assert!(synk_lib::metric_repository::classify("body_composition", "weight").is_some());
    assert!(synk_lib::metric_repository::classify("body_composition", "unknown").is_none());
}

#[test]
fn only_heart_rate_is_split() {
    use synk_lib::metric_repository::classify;
    assert_eq!(
        classify("vital_bp", "heart_rate"),
        Some(MetricSource::Split(LegacyTable::BloodPressure))
    );
    assert_eq!(
        classify("vital_bp", "systolic"),
        Some(MetricSource::Legacy(LegacyTable::BloodPressure))
    );
    assert!(classify("vital_bp", "pulse").is_none());
    assert_eq!(classify("derived", "ffmi"), Some(MetricSource::Metrics));
}
