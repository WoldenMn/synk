use sqlx::SqlitePool;
use synk_lib::commands::alerts::run_alert_checks;

async fn insert_alert(
    pool: &SqlitePool,
    metric_key: &str,
    condition: &str,
    threshold: f64,
    enabled: bool,
) {
    sqlx::query(
        "INSERT INTO health_alerts (metric_key, condition, threshold, label, enabled) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(metric_key)
    .bind(condition)
    .bind(threshold)
    .bind(format!("{} {} {}", metric_key, condition, threshold))
    .bind(if enabled { 1 } else { 0 })
    .execute(pool)
    .await
    .unwrap();
}

async fn insert_weight(pool: &SqlitePool, value: f64) {
    let measured_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    sqlx::query(
        "INSERT INTO body_measures_legacy (measured_at, source, weight) VALUES (?, 'manual', ?)",
    )
    .bind(measured_at)
    .bind(value)
    .execute(pool)
    .await
    .unwrap();
}

async fn insert_systolic(pool: &SqlitePool, value: i64) {
    let measured_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    sqlx::query(
        "INSERT INTO blood_pressure (measured_at, source, systolic, diastolic) \
         VALUES (?, 'manual', ?, 80)",
    )
    .bind(measured_at)
    .bind(value)
    .execute(pool)
    .await
    .unwrap();
}

async fn insert_sleep_score(pool: &SqlitePool, value: i64) {
    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    sqlx::query("INSERT INTO sleep_records (date, source, sleep_score) VALUES (?, 'manual', ?)")
        .bind(date)
        .bind(value)
        .execute(pool)
        .await
        .unwrap();
}

async fn insert_steps(pool: &SqlitePool, value: i64) {
    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    sqlx::query("INSERT INTO activity (date, source, steps) VALUES (?, 'manual', ?)")
        .bind(date)
        .bind(value)
        .execute(pool)
        .await
        .unwrap();
}

#[tokio::test]
async fn threshold_exceeded_creates_event() {
    let pool = crate::common::memory_pool().await;
    insert_alert(&pool, "weight", "above", 80.0, true).await;
    insert_weight(&pool, 85.0).await;

    let events = run_alert_checks(&pool).await.unwrap();
    assert!(!events.is_empty(), "expected at least one alert event");

    let db_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM alert_events")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(db_count.0 >= 1, "event must be persisted in alert_events");
}

#[tokio::test]
async fn an_alert_event_is_stamped_in_utc_with_a_z() {
    let pool = crate::common::memory_pool().await;
    insert_alert(&pool, "weight", "above", 80.0, true).await;
    insert_weight(&pool, 85.0).await;

    let events = run_alert_checks(&pool).await.unwrap();

    let stored: String = sqlx::query_scalar("SELECT triggered_at FROM alert_events")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(stored.ends_with('Z'), "{stored}");
    assert_eq!(events[0].triggered_at, stored);
    assert!(chrono::DateTime::parse_from_rfc3339(&stored).is_ok());
}

#[tokio::test]
async fn threshold_not_exceeded_no_event() {
    let pool = crate::common::memory_pool().await;
    insert_alert(&pool, "weight", "above", 80.0, true).await;
    insert_weight(&pool, 75.0).await;

    let events = run_alert_checks(&pool).await.unwrap();
    assert_eq!(events.len(), 0, "no event when value under threshold");

    let db_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM alert_events")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(db_count.0, 0, "no persisted event expected");
}

#[tokio::test]
async fn inactive_alert_does_not_trigger() {
    let pool = crate::common::memory_pool().await;
    insert_alert(&pool, "weight", "above", 80.0, false).await;
    insert_weight(&pool, 90.0).await;

    let events = run_alert_checks(&pool).await.unwrap();
    assert_eq!(events.len(), 0, "inactive alert must not fire");

    let db_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM alert_events")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(db_count.0, 0, "no persisted event expected");
}

/// V2.7 : systolic est desormais dans le stack V2 via vital_bp (migration 009).
/// L'alerte passe par read_metric_latest qui lit `blood_pressure` via le
/// helper legacy_sql_latest_for_category("vital_bp", "systolic").
/// Test conserve son nom V2.6.1 pour traceabilite git/changelog.
#[tokio::test]
async fn systolic_alert_triggers_via_legacy_query() {
    let pool = crate::common::memory_pool().await;
    insert_alert(&pool, "systolic", "above", 140.0, true).await;
    insert_systolic(&pool, 160).await;

    let events = run_alert_checks(&pool).await.unwrap();
    assert!(
        !events.is_empty(),
        "Alerte systolic doit se declencher (branche legacy)"
    );

    let event = &events[0];
    assert_eq!(event.value, 160.0, "Valeur reportee doit etre 160");

    let db_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM alert_events")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(db_count.0, 1, "1 event persiste");
}

/// V2.7 : sleep_score est desormais dans le stack V2 via vital_sleep.
/// L'alerte doit se declencher via le helper read_metric_latest (plus de hardcode).
#[tokio::test]
async fn sleep_score_alert_triggers_via_v2_stack() {
    let pool = crate::common::memory_pool().await;
    insert_alert(&pool, "sleep_score", "below", 50.0, true).await;
    insert_sleep_score(&pool, 40).await;

    let events = run_alert_checks(&pool).await.unwrap();
    assert!(!events.is_empty(), "Alerte sleep_score doit se declencher");

    let event = &events[0];
    assert_eq!(event.value, 40.0);
}

/// V2.7 : steps est desormais dans le stack V2 via vital_activity.
#[tokio::test]
async fn steps_alert_triggers_via_v2_stack() {
    let pool = crate::common::memory_pool().await;
    insert_alert(&pool, "steps", "below", 5000.0, true).await;
    insert_steps(&pool, 3000).await;

    let events = run_alert_checks(&pool).await.unwrap();
    assert!(!events.is_empty(), "Alerte steps doit se declencher");

    let event = &events[0];
    assert_eq!(event.value, 3000.0);
}

#[tokio::test]
async fn a_database_error_while_reading_the_metric_is_reported_not_swallowed() {
    let pool = crate::common::memory_pool().await;
    insert_alert(&pool, "weight", "above", 80.0, true).await;
    insert_weight(&pool, 90.0).await;
    sqlx::query("ALTER TABLE metric_source_priority RENAME TO metric_source_priority_gone")
        .execute(&pool)
        .await
        .unwrap();

    let result = run_alert_checks(&pool).await;

    assert!(result.is_err(), "l'alerte n'a pas ete evaluee : {result:?}");
}
