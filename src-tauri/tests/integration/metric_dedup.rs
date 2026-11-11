use sqlx::sqlite::SqlitePoolOptions;

async fn pool_with_schema() -> sqlx::SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(":memory:")
        .await
        .unwrap();
    synk_lib::db::bootstrap_schema(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn first_day_of_a_date_column_is_in_the_series() {
    let pool = pool_with_schema().await;
    for (date, score) in [("2026-09-02", 70), ("2026-09-03", 80), ("2026-09-04", 90)] {
        sqlx::query(
            "INSERT INTO sleep_records (date, source, sleep_score) VALUES (?1, 'withings', ?2)",
        )
        .bind(date)
        .bind(score)
        .execute(&pool)
        .await
        .unwrap();
    }

    let pts = synk_lib::metric_repository::read_metric_series(
        &pool,
        "sleep_score",
        "2026-09-03T00:00:00+02:00",
        "2026-09-04T23:59:59+02:00",
    )
    .await
    .unwrap();

    let dates: Vec<&str> = pts.iter().map(|p| p.measured_at.as_str()).collect();
    assert_eq!(dates, vec!["2026-09-03", "2026-09-04"]);
}

#[tokio::test]
async fn last_day_is_in_the_series_when_the_bound_is_a_date() {
    let pool = pool_with_schema().await;
    sqlx::query(
        "INSERT INTO body_measures_legacy (measured_at, source, withings_grpid, weight) VALUES ('2026-06-30T08:00:00Z', 'withings', 'g9', 71.0)",
    )
    .execute(&pool)
    .await
    .unwrap();

    let pts = synk_lib::metric_repository::read_metric_series(
        &pool,
        "weight",
        "2026-06-01",
        "2026-06-30",
    )
    .await
    .unwrap();

    assert_eq!(
        pts.len(),
        1,
        "la pesee du 30 juin a 8 h appartient a la plage qui finit le 30 juin"
    );
}

#[tokio::test]
async fn range_follows_the_utc_day_like_select_range() {
    let pool = pool_with_schema().await;
    for (at, ext) in [("2026-09-02T22:30:00Z", "a"), ("2026-09-03T06:00:00Z", "b")] {
        sqlx::query(
            "INSERT INTO body_measures_legacy (measured_at, source, withings_grpid, weight) VALUES (?1, 'withings', ?2, 70.0)",
        )
        .bind(at)
        .bind(ext)
        .execute(&pool)
        .await
        .unwrap();
    }

    let pts = synk_lib::metric_repository::read_metric_series(
        &pool,
        "weight",
        "2026-09-03T00:00:00+02:00",
        "2026-09-03T23:59:59+02:00",
    )
    .await
    .unwrap();

    let at: Vec<&str> = pts.iter().map(|p| p.measured_at.as_str()).collect();
    assert_eq!(at, vec!["2026-09-03T06:00:00Z"]);
}
