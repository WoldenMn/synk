use sqlx::SqlitePool;
use synk_lib::services::insights::detectors::{
    heart_rate_anomaly, hypertension, sleep_insufficient,
};
use synk_lib::services::metrics::set_priority;

use crate::common::memory_pool;

fn iso(days_ago: i64) -> String {
    (chrono::Utc::now() - chrono::Duration::days(days_ago))
        .format("%Y-%m-%dT12:00:00Z")
        .to_string()
}

fn day(days_ago: i64) -> String {
    (chrono::Utc::now() - chrono::Duration::days(days_ago))
        .format("%Y-%m-%d")
        .to_string()
}

async fn night(pool: &SqlitePool, source: &str, days_ago: i64, seconds: i64) {
    sqlx::query("INSERT INTO sleep_records (date, source, duration_total) VALUES (?1, ?2, ?3)")
        .bind(day(days_ago))
        .bind(source)
        .bind(seconds)
        .execute(pool)
        .await
        .unwrap();
}

async fn pressure(pool: &SqlitePool, source: &str, days_ago: i64, sys: i64, dia: i64) {
    sqlx::query(
        "INSERT INTO blood_pressure (measured_at, source, systolic, diastolic) VALUES (?1, ?2, ?3, ?4)",
    )
    .bind(iso(days_ago))
    .bind(source)
    .bind(sys)
    .bind(dia)
    .execute(pool)
    .await
    .unwrap();
}

async fn ecg(pool: &SqlitePool, source: &str, days_ago: i64, hour: u32, bpm: i64) {
    let at = (chrono::Utc::now() - chrono::Duration::days(days_ago))
        .format(&format!("%Y-%m-%dT{hour:02}:00:00Z"))
        .to_string();
    sqlx::query("INSERT INTO heart_records (timestamp, source, heart_rate) VALUES (?1, ?2, ?3)")
        .bind(at)
        .bind(source)
        .bind(bpm)
        .execute(pool)
        .await
        .unwrap();
}

#[tokio::test]
async fn two_sources_for_the_same_nights_do_not_make_three_short_nights() {
    let pool = memory_pool().await;
    for days_ago in [1, 2] {
        night(&pool, "withings", days_ago, 18_000).await;
        night(&pool, "oura", days_ago, 19_000).await;
    }

    assert_eq!(sleep_insufficient::detect(&pool).await.unwrap(), 0);
}

#[tokio::test]
async fn three_short_nights_still_raise_the_insight_when_two_sources_overlap() {
    let pool = memory_pool().await;
    for days_ago in [1, 2, 3] {
        night(&pool, "withings", days_ago, 18_000).await;
    }
    night(&pool, "oura", 1, 19_000).await;

    assert_eq!(sleep_insufficient::detect(&pool).await.unwrap(), 1);
}

#[tokio::test]
async fn a_reading_from_the_source_the_user_ranked_lower_is_not_the_last_one() {
    let pool = memory_pool().await;
    set_priority(&pool, "systolic", &["withings", "manual"])
        .await
        .unwrap();
    pressure(&pool, "withings", 3, 118, 76).await;
    pressure(&pool, "withings", 1, 119, 77).await;
    pressure(&pool, "manual", 1, 150, 95).await;

    assert_eq!(hypertension::detect(&pool).await.unwrap(), 0);
}

#[tokio::test]
async fn a_high_reading_from_the_chosen_source_still_raises_the_insight() {
    let pool = memory_pool().await;
    set_priority(&pool, "systolic", &["manual", "withings"])
        .await
        .unwrap();
    pressure(&pool, "withings", 3, 118, 76).await;
    pressure(&pool, "withings", 1, 119, 77).await;
    pressure(&pool, "manual", 1, 150, 95).await;

    assert_eq!(hypertension::detect(&pool).await.unwrap(), 1);
}

#[tokio::test]
async fn an_ecg_from_the_source_ranked_lower_does_not_make_an_anomaly() {
    let pool = memory_pool().await;
    set_priority(&pool, "heart_rate", &["withings", "manual"])
        .await
        .unwrap();
    for (i, bpm) in [55, 75, 55, 75, 55, 75, 55, 75, 55, 75]
        .into_iter()
        .enumerate()
    {
        ecg(&pool, "withings", i as i64 + 2, 12, bpm).await;
    }
    ecg(&pool, "withings", 1, 12, 76).await;
    ecg(&pool, "manual", 1, 18, 150).await;

    assert_eq!(heart_rate_anomaly::detect(&pool).await.unwrap(), 0);
}
