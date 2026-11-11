#[tokio::test]
async fn temperature_definition_exists_after_bootstrap() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect(":memory:")
        .await
        .unwrap();
    synk_lib::db::bootstrap_schema(&pool).await.unwrap();
    let cat: Option<String> =
        sqlx::query_scalar("SELECT category FROM metric_definitions WHERE slug='temperature'")
            .fetch_optional(&pool)
            .await
            .unwrap();
    assert_eq!(cat.as_deref(), Some("body_scan"));
}
