use sqlx::SqlitePool;
use synk_lib::commands::stats::run_metric_trend;

async fn insert_weight(pool: &SqlitePool, date: &str, weight: f64) {
    sqlx::query(
        "INSERT INTO body_measures_legacy (measured_at, weight, source) VALUES (?, ?, 'manual')",
    )
    .bind(date)
    .bind(weight)
    .execute(pool)
    .await
    .unwrap();
}

#[tokio::test]
async fn trend_up_when_current_higher_than_previous() {
    let pool = crate::common::memory_pool().await;
    insert_weight(&pool, "2026-03-15T08:00:00Z", 70.0).await;
    insert_weight(&pool, "2026-04-10T08:00:00Z", 73.0).await;

    let trend = run_metric_trend(&pool, "weight", "2026-04-01", "2026-04-30")
        .await
        .unwrap();

    assert_eq!(trend.current_value, Some(73.0));
    assert_eq!(trend.previous_value, Some(70.0));
    assert_eq!(trend.direction, "up");
    let pct = trend.percent_change.unwrap();
    assert!((pct - 4.2857).abs() < 0.01, "got {}", pct);
}

#[tokio::test]
async fn trend_down_when_current_lower_than_previous() {
    let pool = crate::common::memory_pool().await;
    insert_weight(&pool, "2026-03-15T08:00:00Z", 75.0).await;
    insert_weight(&pool, "2026-04-10T08:00:00Z", 72.0).await;

    let trend = run_metric_trend(&pool, "weight", "2026-04-01", "2026-04-30")
        .await
        .unwrap();

    assert_eq!(trend.direction, "down");
    assert!(trend.percent_change.unwrap() < 0.0);
}

#[tokio::test]
async fn trend_current_value_includes_a_weighing_on_the_last_day() {
    let pool = crate::common::memory_pool().await;
    insert_weight(&pool, "2026-04-10T08:00:00Z", 73.0).await;
    insert_weight(&pool, "2026-04-30T07:15:00Z", 71.5).await;

    let trend = run_metric_trend(&pool, "weight", "2026-04-01", "2026-04-30T19:00:00+02:00")
        .await
        .unwrap();

    assert_eq!(trend.current_value, Some(71.5));
}

#[tokio::test]
async fn trend_stable_when_no_data() {
    let pool = crate::common::memory_pool().await;
    let trend = run_metric_trend(&pool, "weight", "2026-04-01", "2026-04-30")
        .await
        .unwrap();
    assert_eq!(trend.current_value, None);
    assert_eq!(trend.previous_value, None);
    assert_eq!(trend.direction, "stable");
    assert_eq!(trend.percent_change, None);
}

#[tokio::test]
async fn returns_err_for_unknown_metric() {
    let pool = crate::common::memory_pool().await;
    let res = run_metric_trend(&pool, "not_a_metric", "2026-04-01", "2026-04-30").await;
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.kind(), "Validation");
    assert!(err.detail().contains("Tendance non disponible"));
}

#[tokio::test]
async fn retired_trend_branches_are_rejected() {
    let pool = crate::common::memory_pool().await;
    for metric in [
        "workout_count",
        "fat_ratio",
        "muscle_mass",
        "diastolic",
        "heart_rate",
        "calories_active",
    ] {
        let res = run_metric_trend(&pool, metric, "2026-01-01", "2026-01-31").await;
        let err = res.expect_err(metric);
        assert_eq!(err.kind(), "Validation", "{metric}");
        assert!(
            err.detail().contains("Tendance non disponible"),
            "{metric} : {err}"
        );
    }
}

#[tokio::test]
async fn the_four_dashboard_trends_are_still_served() {
    let pool = crate::common::memory_pool().await;
    for metric in ["weight", "systolic", "sleep_score", "steps"] {
        let res = run_metric_trend(&pool, metric, "2026-01-01", "2026-01-31").await;
        assert!(res.is_ok(), "{metric}");
    }
}
