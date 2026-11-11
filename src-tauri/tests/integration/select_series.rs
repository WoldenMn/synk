use chrono::{TimeZone, Utc};
use sqlx::SqlitePool;
use synk_lib::services::metrics::{
    get_priority, select_latest, select_latest_before, select_series, set_priority,
    write_observations,
};
use synk_lib::sources::{Observation, SourceId};

use crate::common::memory_pool;

const FROM: &str = "2026-01-01";
const TO: &str = "2026-12-31";

async fn weigh(pool: &SqlitePool, source: &str, at: &str, kg: f64) {
    sqlx::query(
        "INSERT INTO body_measures_legacy (measured_at, source, weight) VALUES (?1, ?2, ?3)",
    )
    .bind(at)
    .bind(source)
    .bind(kg)
    .execute(pool)
    .await
    .unwrap();
}

async fn steps(pool: &SqlitePool, source: &str, date: &str, n: i64) {
    sqlx::query("INSERT INTO activity (date, source, steps) VALUES (?1, ?2, ?3)")
        .bind(date)
        .bind(source)
        .bind(n)
        .execute(pool)
        .await
        .unwrap();
}

async fn rows(pool: &SqlitePool, table: &str) -> i64 {
    sqlx::query_scalar(sqlx::AssertSqlSafe(format!("SELECT COUNT(*) FROM {table}")))
        .fetch_one(pool)
        .await
        .unwrap()
}

fn observation(
    metric: &str,
    source: &'static str,
    external_id: Option<&str>,
    value: f64,
) -> Observation {
    Observation {
        metric: metric.to_string(),
        measured_at: Utc.with_ymd_and_hms(2026, 3, 1, 7, 30, 0).unwrap(),
        value,
        unit: String::new(),
        source: SourceId::new(source),
        external_id: external_id.map(str::to_string),
        device: None,
    }
}

#[tokio::test]
async fn two_weights_the_same_morning_show_one_value_and_keep_two_rows() {
    let pool = memory_pool().await;
    weigh(&pool, "withings", "2026-03-01T07:00:00Z", 80.0).await;
    weigh(&pool, "csv", "2026-03-01T07:30:00Z", 80.4).await;

    let series = select_series(&pool, "weight", FROM, TO).await.unwrap();

    assert_eq!(series.len(), 1, "{series:?}");
    assert_eq!(series[0].source, "withings");
    assert_eq!(series[0].value, 80.0);
    assert_eq!(rows(&pool, "body_measures_legacy").await, 2);
}

#[tokio::test]
async fn by_default_the_source_with_the_most_recent_point_wins_the_shared_days() {
    let pool = memory_pool().await;
    weigh(&pool, "withings", "2026-03-01T07:00:00Z", 80.0).await;
    weigh(&pool, "withings", "2026-03-02T07:00:00Z", 79.0).await;
    weigh(&pool, "csv", "2026-03-02T07:30:00Z", 79.4).await;
    weigh(&pool, "csv", "2026-03-03T07:30:00Z", 79.0).await;

    let series = select_series(&pool, "weight", FROM, TO).await.unwrap();
    let seen: Vec<(&str, &str)> = series
        .iter()
        .map(|p| (&p.measured_at[..10], p.source.as_str()))
        .collect();

    assert_eq!(
        seen,
        vec![
            ("2026-03-01", "withings"),
            ("2026-03-02", "csv"),
            ("2026-03-03", "csv")
        ],
        "csv a le point le plus recent : il prend les jours communs, withings garde les siens"
    );
}

async fn connected(pool: &SqlitePool, source: &str, at: &str) {
    sqlx::query(
        "INSERT INTO source_auth (source_id, access_token, refresh_token, expires_at, created_at) VALUES (?1, 'x', 'x', 0, ?2)",
    )
    .bind(source)
    .bind(at)
    .execute(pool)
    .await
    .unwrap();
}

async fn winner_of_a_shared_day(pool: &SqlitePool, first: &str, second: &str) -> String {
    weigh(pool, first, "2026-03-02T07:00:00Z", 80.0).await;
    weigh(pool, second, "2026-03-02T07:30:00Z", 80.4).await;
    let series = select_series(pool, "weight", FROM, TO).await.unwrap();
    assert_eq!(series.len(), 1);
    series[0].source.clone()
}

#[tokio::test]
async fn on_a_tie_the_source_connected_first_wins_whatever_its_registry_rank() {
    let pool = memory_pool().await;
    connected(&pool, "polar", "2026-02-01 08:00:00").await;
    connected(&pool, "oura", "2026-03-01 08:00:00").await;
    assert_eq!(
        winner_of_a_shared_day(&pool, "oura", "polar").await,
        "polar"
    );
}

#[tokio::test]
async fn the_registry_order_still_breaks_a_tie_between_two_connections_of_the_same_second() {
    let pool = memory_pool().await;
    connected(&pool, "polar", "2026-02-01 08:00:00").await;
    connected(&pool, "oura", "2026-02-01 08:00:00").await;
    assert_eq!(winner_of_a_shared_day(&pool, "polar", "oura").await, "oura");
}

#[tokio::test]
async fn a_connected_source_comes_before_a_file_source_on_a_tie() {
    let pool = memory_pool().await;
    connected(&pool, "oura", "2026-02-01 08:00:00").await;
    assert_eq!(
        winner_of_a_shared_day(&pool, "import", "oura").await,
        "oura"
    );
}

#[tokio::test]
async fn a_source_disconnected_and_connected_again_takes_its_new_place() {
    let pool = memory_pool().await;
    connected(&pool, "oura", "2026-02-01 08:00:00").await;
    connected(&pool, "polar", "2026-02-15 08:00:00").await;
    synk_lib::sources::store::delete_auth(&pool, SourceId::OURA)
        .await
        .unwrap();
    connected(&pool, "oura", "2026-03-20 08:00:00").await;
    assert_eq!(
        winner_of_a_shared_day(&pool, "oura", "polar").await,
        "polar"
    );
}

#[tokio::test]
async fn two_step_totals_are_never_added_up() {
    let pool = memory_pool().await;
    steps(&pool, "withings", "2026-03-01", 8000).await;
    steps(&pool, "fit", "2026-03-01", 12000).await;

    let series = select_series(&pool, "steps", FROM, TO).await.unwrap();

    assert_eq!(series.len(), 1, "{series:?}");
    assert_eq!(series[0].value, 8000.0);
    assert_eq!(rows(&pool, "activity").await, 2);
}

#[tokio::test]
async fn changing_the_priority_changes_the_curve() {
    let pool = memory_pool().await;
    steps(&pool, "withings", "2026-03-01", 8000).await;
    steps(&pool, "fit", "2026-03-01", 12000).await;

    set_priority(&pool, "steps", &["fit", "withings"])
        .await
        .unwrap();
    let fit_first = select_series(&pool, "steps", FROM, TO).await.unwrap();
    assert_eq!(fit_first[0].value, 12000.0);
    assert_eq!(fit_first[0].source, "fit");

    set_priority(&pool, "steps", &["withings", "fit"])
        .await
        .unwrap();
    let withings_first = select_series(&pool, "steps", FROM, TO).await.unwrap();
    assert_eq!(withings_first[0].value, 8000.0);
    assert_eq!(
        get_priority(&pool, "steps").await.unwrap(),
        vec!["withings".to_string(), "fit".to_string()]
    );
}

#[tokio::test]
async fn a_priority_source_without_data_that_day_leaves_the_day_to_the_next_one() {
    let pool = memory_pool().await;
    steps(&pool, "withings", "2026-03-01", 8000).await;
    steps(&pool, "withings", "2026-03-02", 9000).await;
    steps(&pool, "fit", "2026-03-02", 12000).await;
    set_priority(&pool, "steps", &["fit"]).await.unwrap();

    let series = select_series(&pool, "steps", FROM, TO).await.unwrap();
    let values: Vec<f64> = series.iter().map(|p| p.value).collect();

    assert_eq!(values, vec![8000.0, 12000.0]);
}

#[tokio::test]
async fn a_reimport_adds_nothing() {
    let pool = memory_pool().await;
    let batch = [
        observation("weight", "csv", Some("renpho-1"), 80.4),
        observation("steps", "fit", Some("fit-day-1"), 12000.0),
        observation("heart_rate", "apple_health", Some("hr-1"), 61.0),
        observation("spo2", "apple_health", None, 97.0),
    ];

    let first = write_observations(&pool, &batch).await.unwrap();
    let second = write_observations(&pool, &batch).await.unwrap();

    assert_eq!((first.inserted, first.ignored), (4, 0));
    assert_eq!((second.inserted, second.ignored), (0, 4));
    assert_eq!(rows(&pool, "body_measures_legacy").await, 1);
    assert_eq!(rows(&pool, "activity").await, 1);
    assert_eq!(rows(&pool, "metrics").await, 1);
    assert_eq!(rows(&pool, "vitals_legacy").await, 1);
}

#[tokio::test]
async fn a_metric_with_no_single_column_to_land_in_is_refused_not_guessed() {
    let pool = memory_pool().await;
    let err = write_observations(&pool, &[observation("systolic", "csv", Some("a"), 120.0)])
        .await
        .unwrap_err();
    assert_eq!(err.kind(), "Validation");
    let err = write_observations(&pool, &[observation("inconnue", "csv", Some("a"), 1.0)])
        .await
        .unwrap_err();
    assert_eq!(err.kind(), "Validation");
    assert_eq!(rows(&pool, "blood_pressure").await, 0);
}

#[tokio::test]
async fn the_latest_point_follows_the_same_choice() {
    let pool = memory_pool().await;
    steps(&pool, "withings", "2026-03-01", 8000).await;
    steps(&pool, "withings", "2026-03-02", 9000).await;
    steps(&pool, "fit", "2026-03-02", 12000).await;

    let latest = select_latest(&pool, "steps").await.unwrap().unwrap();
    assert_eq!((latest.value, latest.source.as_str()), (9000.0, "withings"));

    set_priority(&pool, "steps", &["fit"]).await.unwrap();
    let latest = select_latest(&pool, "steps").await.unwrap().unwrap();
    assert_eq!((latest.value, latest.source.as_str()), (12000.0, "fit"));

    let before = select_latest_before(&pool, "steps", "2026-03-02")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(before.value, 8000.0);
    assert!(select_latest_before(&pool, "steps", "2026-03-01")
        .await
        .unwrap()
        .is_none());
}

#[tokio::test]
async fn heart_rate_keeps_its_two_tables_and_still_picks_one_source_per_day() {
    let pool = memory_pool().await;
    sqlx::query("INSERT INTO blood_pressure (measured_at, source, systolic, diastolic, heart_rate) VALUES ('2026-03-01T08:00:00Z', 'withings', 120, 80, 60)")
        .execute(&pool)
        .await
        .unwrap();
    write_observations(
        &pool,
        &[observation("heart_rate", "apple_health", Some("h1"), 65.0)],
    )
    .await
    .unwrap();
    sqlx::query("INSERT INTO blood_pressure (measured_at, source, systolic, diastolic, heart_rate) VALUES ('2026-03-05T08:00:00Z', 'manual', 118, 78, 58)")
        .execute(&pool)
        .await
        .unwrap();

    let series = select_series(&pool, "heart_rate", FROM, TO).await.unwrap();
    let seen: Vec<(&str, &str)> = series
        .iter()
        .map(|p| (&p.measured_at[..10], p.source.as_str()))
        .collect();

    assert_eq!(
        seen,
        vec![("2026-03-01", "withings"), ("2026-03-05", "manual")]
    );
}

#[tokio::test]
async fn an_unknown_slug_is_not_found_for_a_series_and_has_no_latest() {
    let pool = memory_pool().await;
    assert_eq!(
        select_series(&pool, "inconnue", FROM, TO)
            .await
            .unwrap_err()
            .kind(),
        "NotFound"
    );
    assert!(select_latest(&pool, "inconnue").await.unwrap().is_none());
    assert!(set_priority(&pool, "inconnue", &["withings"])
        .await
        .is_err());
}

#[tokio::test]
async fn a_priority_naming_an_unknown_source_is_refused() {
    let pool = crate::common::memory_pool_with_foreign_keys().await;
    let err = set_priority(&pool, "steps", &["withings", "inconnue"])
        .await
        .unwrap_err();
    assert_eq!(err.kind(), "Validation", "{err}");
    assert!(get_priority(&pool, "steps").await.unwrap().is_empty());
}

#[tokio::test]
#[ignore = "lit une copie migree de la base de dev : SYNK_PROOF_COPY=<dossier de synk.db>"]
async fn select_series_against_the_raw_series_on_a_dev_copy() {
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    let dir = std::env::var("SYNK_PROOF_COPY").expect("SYNK_PROOF_COPY");
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(format!("{dir}/synk.db"))
                .read_only(true),
        )
        .await
        .unwrap();
    let slugs: Vec<String> =
        sqlx::query_scalar("SELECT slug FROM metric_definitions ORDER BY slug")
            .fetch_all(&pool)
            .await
            .unwrap();
    let mut removed_total = 0;
    for slug in &slugs {
        let raw = synk_lib::metric_repository::read_metric_series(
            &pool,
            slug,
            "2000-01-01",
            "2100-01-01",
        )
        .await
        .unwrap();
        let chosen = select_series(&pool, slug, "2000-01-01", "2100-01-01")
            .await
            .unwrap();
        let removed = raw.len() - chosen.len();
        removed_total += removed;
        let multi: std::collections::BTreeSet<&str> =
            raw.iter().map(|p| p.source.as_str()).collect();
        if removed > 0 || multi.len() > 1 {
            println!(
                "{slug} : brut {} choisi {} retires {removed} sources {multi:?}",
                raw.len(),
                chosen.len()
            );
        }
    }
    println!(
        "{} slugs, points retires au total : {removed_total}",
        slugs.len()
    );
}
