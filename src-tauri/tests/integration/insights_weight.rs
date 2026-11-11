use sqlx::SqlitePool;
use synk_lib::services::insights::run_generation;

async fn insert_weight(pool: &SqlitePool, date: &str, value: f64) {
    sqlx::query("INSERT INTO body_measures_legacy (measured_at, weight) VALUES (?, ?)")
        .bind(date)
        .bind(value)
        .execute(pool)
        .await
        .unwrap();
}

#[tokio::test]
async fn detects_weight_increase_trend() {
    let pool = crate::common::memory_pool().await;
    let today = chrono::Utc::now();
    for i in 0..14 {
        let d = today - chrono::Duration::days(14 - i);
        let date = d.format("%Y-%m-%dT%H:%M:%SZ").to_string();
        insert_weight(&pool, &date, 70.0 + (i as f64) * 0.2).await;
    }

    let count = run_generation(&pool).await;
    assert!(
        count.generated >= 1,
        "expected at least one insight generated"
    );

    let row: (String, String, Option<String>) = sqlx::query_as(
        "SELECT kind, severity, metric FROM insights WHERE metric = 'weight' LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(row.0, "trend");
    assert_eq!(row.1, "warning");
    assert_eq!(row.2, Some("weight".to_string()));
}

#[tokio::test]
async fn detects_weight_decrease_trend() {
    let pool = crate::common::memory_pool().await;
    let today = chrono::Utc::now();
    for i in 0..14 {
        let d = today - chrono::Duration::days(14 - i);
        let date = d.format("%Y-%m-%dT%H:%M:%SZ").to_string();
        insert_weight(&pool, &date, 80.0 - (i as f64) * 0.2).await;
    }

    run_generation(&pool).await;

    let row: (String,) =
        sqlx::query_as("SELECT severity FROM insights WHERE metric = 'weight' LIMIT 1")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(row.0, "info");
}

#[tokio::test]
async fn no_insight_when_weight_stable() {
    let pool = crate::common::memory_pool().await;
    let today = chrono::Utc::now();
    for i in 0..14 {
        let d = today - chrono::Duration::days(14 - i);
        let date = d.format("%Y-%m-%dT%H:%M:%SZ").to_string();
        insert_weight(&pool, &date, 70.0).await;
    }

    run_generation(&pool).await;

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM insights WHERE metric = 'weight'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count.0, 0, "no insight expected when delta < 0.5kg");
}
