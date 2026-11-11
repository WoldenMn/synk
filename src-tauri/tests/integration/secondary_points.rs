use sqlx::SqlitePool;
use synk_lib::services::metrics::{select_secondary, select_series, set_priority};

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

#[tokio::test]
async fn the_point_the_curve_does_not_use_stays_available_as_a_secondary_one() {
    let pool = memory_pool().await;
    weigh(&pool, "withings", "2026-03-01T07:00:00Z", 80.0).await;
    weigh(&pool, "manual", "2026-03-01T09:00:00Z", 79.0).await;
    weigh(&pool, "withings", "2026-03-02T07:00:00Z", 79.5).await;

    let chosen = select_series(&pool, "weight", FROM, TO).await.unwrap();
    let others = select_secondary(&pool, "weight", FROM, TO).await.unwrap();

    assert_eq!(chosen.len(), 2);
    assert_eq!(others.len(), 1);
    assert_ne!(others[0].source, chosen[0].source);
    assert_eq!(others[0].measured_at, "2026-03-01T09:00:00Z");
    assert_eq!(others[0].source, "manual");
}

#[tokio::test]
async fn a_daily_total_of_another_source_is_shown_not_added() {
    let pool = memory_pool().await;
    steps(&pool, "withings", "2026-03-01", 8000).await;
    steps(&pool, "oura", "2026-03-01", 9500).await;
    set_priority(&pool, "steps", &["withings", "oura"])
        .await
        .unwrap();

    let chosen = select_series(&pool, "steps", FROM, TO).await.unwrap();
    let others = select_secondary(&pool, "steps", FROM, TO).await.unwrap();

    assert_eq!(
        chosen.iter().map(|p| p.value).collect::<Vec<_>>(),
        vec![8000.0]
    );
    assert_eq!(
        others.iter().map(|p| p.value).collect::<Vec<_>>(),
        vec![9500.0]
    );
}

#[tokio::test]
async fn one_source_alone_has_no_secondary_point() {
    let pool = memory_pool().await;
    weigh(&pool, "withings", "2026-03-01T07:00:00Z", 80.0).await;

    assert!(select_secondary(&pool, "weight", FROM, TO)
        .await
        .unwrap()
        .is_empty());
    assert_eq!(
        select_secondary(&pool, "inconnue", FROM, TO)
            .await
            .unwrap_err()
            .kind(),
        "NotFound"
    );
}
