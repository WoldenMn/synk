use sqlx::sqlite::SqlitePoolOptions;

#[tokio::test]
async fn spo2_bounds_widened_after_bootstrap() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(":memory:")
        .await
        .unwrap();
    synk_lib::db::bootstrap_schema(&pool).await.unwrap();
    let (min, max): (f64, f64) =
        sqlx::query_as("SELECT min_value, max_value FROM metric_definitions WHERE slug='spo2'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!((min, max), (0.0, 100.0));
    let (tmin, tmax): (f64, f64) = sqlx::query_as(
        "SELECT min_value, max_value FROM metric_definitions WHERE slug='body_temperature'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((tmin, tmax), (0.0, 60.0));
}
