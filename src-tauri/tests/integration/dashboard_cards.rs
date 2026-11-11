use serde_json::json;
use synk_lib::commands::dashboard::dashboard_cards;
use synk_lib::commands::stats::run_metric_trend;

async fn seed(pool: &sqlx::SqlitePool) {
    for sql in [
        "INSERT INTO body_measures_legacy (measured_at, weight, source) VALUES ('2026-03-15T08:00:00Z', 70.0, 'manual')",
        "INSERT INTO body_measures_legacy (measured_at, weight, source) VALUES ('2026-04-10T08:00:00Z', 73.0, 'manual')",
        "INSERT INTO blood_pressure (measured_at, source, withings_grpid, systolic, diastolic, heart_rate) VALUES ('2026-04-11T08:00:00Z', 'manual', 'b1', 125, 82, 66)",
        "INSERT INTO sleep_records (date, source, sleep_score) VALUES ('2026-04-12', 'withings', 81)",
        "INSERT INTO activity (date, source, steps) VALUES ('2026-04-13', 'withings', 9100)",
        "INSERT INTO workouts (date, start_at, source, category_label, duration) VALUES ('2026-04-14', '2026-04-14T18:00:00Z', 'manual', 'Course', 1800)",
        "INSERT INTO heart_records (timestamp, source, ecg_status, heart_rate) VALUES ('2026-04-15T09:00:00Z', 'withings', 1, 64)",
    ] {
        sqlx::query(sql).execute(pool).await.expect(sql);
    }
}

#[tokio::test]
async fn the_cards_answer_what_the_ten_separate_reads_answered() {
    let pool = crate::common::memory_pool().await;
    seed(&pool).await;

    let cards = dashboard_cards(&pool, "2026-04-01", "2026-04-30")
        .await
        .unwrap();

    let value = serde_json::to_value(&cards).unwrap();
    assert_eq!(value["body"]["weight"], json!(73.0));
    assert_eq!(value["blood_pressure"]["systolic"], json!(125));
    assert_eq!(value["sleep"]["sleep_score"], json!(81));
    assert_eq!(value["activity"]["steps"], json!(9100));
    assert_eq!(value["workout"]["category_label"], json!("Course"));
    assert_eq!(value["heart"]["heart_rate"], json!(64));
    for (key, metric) in [
        ("weight", "weight"),
        ("systolic", "systolic"),
        ("sleep_score", "sleep_score"),
        ("steps", "steps"),
    ] {
        let alone = run_metric_trend(&pool, metric, "2026-04-01", "2026-04-30")
            .await
            .unwrap();
        assert_eq!(
            value["trends"][key],
            serde_json::to_value(&alone).unwrap(),
            "tendance {key}"
        );
    }
    assert_eq!(value["trends"]["weight"]["previous_value"], json!(70.0));
}

#[tokio::test]
async fn an_empty_base_gives_empty_cards_and_neutral_trends() {
    let pool = crate::common::memory_pool().await;

    let cards = dashboard_cards(&pool, "2026-04-01", "2026-04-30")
        .await
        .unwrap();

    let value = serde_json::to_value(&cards).unwrap();
    for key in [
        "body",
        "blood_pressure",
        "sleep",
        "activity",
        "workout",
        "heart",
    ] {
        assert!(value[key].is_null(), "{key}");
    }
    assert_eq!(value["trends"]["weight"]["direction"], json!("stable"));
}

#[tokio::test]
async fn an_unreadable_end_date_is_a_validation_error() {
    let pool = crate::common::memory_pool().await;
    let err = dashboard_cards(&pool, "2026-04-01", "pas-une-date")
        .await
        .unwrap_err();
    assert_eq!(err.kind(), "Validation");
}

async fn put_two_weighings(pool: &sqlx::SqlitePool) {
    for sql in [
        "INSERT INTO body_measures_legacy (measured_at, weight, source) VALUES ('2026-04-10T08:00:00Z', 73.0, 'withings')",
        "INSERT INTO body_measures_legacy (measured_at, weight, source) VALUES ('2026-04-10T09:00:00Z', 74.0, 'manual')",
        "INSERT INTO blood_pressure (measured_at, source, withings_grpid, systolic, diastolic, heart_rate) VALUES ('2026-04-11T08:00:00Z', 'withings', 'b1', 120, 80, 60)",
        "INSERT INTO blood_pressure (measured_at, source, withings_grpid, systolic, diastolic, heart_rate) VALUES ('2026-04-11T09:00:00Z', 'manual', 'b2', 135, 85, 70)",
        "INSERT INTO sleep_records (date, source, sleep_score) VALUES ('2026-04-12', 'withings', 70)",
        "INSERT INTO sleep_records (date, source, sleep_score) VALUES ('2026-04-12', 'oura', 85)",
        "INSERT INTO activity (date, source, steps) VALUES ('2026-04-13', 'withings', 9100)",
        "INSERT INTO activity (date, source, steps) VALUES ('2026-04-13', 'polar', 4200)",
    ] {
        sqlx::query(sql).execute(pool).await.expect(sql);
    }
}

async fn prefer(pool: &sqlx::SqlitePool, slug: &str, order: &[&str]) {
    synk_lib::services::metrics::set_priority(pool, slug, order)
        .await
        .unwrap();
}

async fn cards_as_json(pool: &sqlx::SqlitePool) -> serde_json::Value {
    let cards = dashboard_cards(pool, "2026-04-01", "2026-04-30")
        .await
        .unwrap();
    serde_json::to_value(&cards).unwrap()
}

#[tokio::test]
async fn the_cards_follow_the_source_priority_the_user_chose() {
    let pool = crate::common::memory_pool().await;
    put_two_weighings(&pool).await;

    prefer(&pool, "weight", &["withings", "manual"]).await;
    prefer(&pool, "systolic", &["withings", "manual"]).await;
    prefer(&pool, "sleep_score", &["oura", "withings"]).await;
    prefer(&pool, "steps", &["polar", "withings"]).await;
    let first = cards_as_json(&pool).await;
    assert_eq!(first["body"]["weight"], json!(73.0));
    assert_eq!(first["blood_pressure"]["systolic"], json!(120));
    assert_eq!(first["sleep"]["sleep_score"], json!(85));
    assert_eq!(first["activity"]["steps"], json!(4200));

    prefer(&pool, "weight", &["manual", "withings"]).await;
    prefer(&pool, "systolic", &["manual", "withings"]).await;
    prefer(&pool, "sleep_score", &["withings", "oura"]).await;
    prefer(&pool, "steps", &["withings", "polar"]).await;
    let second = cards_as_json(&pool).await;
    assert_eq!(second["body"]["weight"], json!(74.0));
    assert_eq!(second["blood_pressure"]["systolic"], json!(135));
    assert_eq!(second["sleep"]["sleep_score"], json!(70));
    assert_eq!(second["activity"]["steps"], json!(9100));
}

#[tokio::test]
async fn a_card_and_its_trend_announce_the_same_value() {
    let pool = crate::common::memory_pool().await;
    put_two_weighings(&pool).await;
    prefer(&pool, "weight", &["withings", "manual"]).await;
    prefer(&pool, "sleep_score", &["withings", "oura"]).await;

    let value = cards_as_json(&pool).await;

    assert_eq!(
        value["body"]["weight"].as_f64(),
        value["trends"]["weight"]["current_value"].as_f64()
    );
    assert_eq!(
        value["sleep"]["sleep_score"].as_f64(),
        value["trends"]["sleep_score"]["current_value"].as_f64()
    );
}

#[tokio::test]
async fn a_card_without_the_pivot_metric_keeps_the_latest_row() {
    let pool = crate::common::memory_pool().await;
    sqlx::query(
        "INSERT INTO body_measures_legacy (measured_at, fat_ratio, source) VALUES ('2026-04-10T08:00:00Z', 21.5, 'manual')",
    )
    .execute(&pool)
    .await
    .unwrap();

    let value = cards_as_json(&pool).await;

    assert_eq!(value["body"]["fat_ratio"], json!(21.5));
}

#[tokio::test]
async fn reading_the_cards_stays_within_its_statement_budget() {
    let counted = crate::common::capture::counted_pool().await;
    put_two_weighings(&counted.pool).await;
    prefer(&counted.pool, "weight", &["withings", "manual"]).await;

    let (_, used) = counted.count(cards_as_json(&counted.pool)).await;

    println!("STATEMENTS {used}");
    assert!(used > 0, "l'instrument ne voit pas les requetes");
    assert!(used <= 32, "{used} requetes pour quatre mesures");
}

#[tokio::test]
async fn the_statement_counter_counts_exactly_what_it_is_given() {
    let counted = crate::common::capture::counted_pool().await;

    let (_, none) = counted.count(async {}).await;
    let (_, three) = counted
        .count(async {
            for _ in 0..3 {
                sqlx::query("SELECT 1")
                    .execute(&counted.pool)
                    .await
                    .unwrap();
            }
        })
        .await;

    assert_eq!((none, three), (0, 3));
}
