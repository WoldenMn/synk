use synk_lib::commands::export::export_csv_inner;

async fn pool_with_one_weight(dir: &tempfile::TempDir) -> sqlx::SqlitePool {
    let pool = synk_lib::db::init_pool(dir.path().join("synk.db"))
        .await
        .expect("base temporaire initialisee");
    sqlx::query(
        "INSERT INTO body_measures_legacy (measured_at, source, weight) VALUES ('2026-09-30T22:00:00Z', 'manual', 72.5)",
    )
    .execute(&pool)
    .await
    .unwrap();
    pool
}

fn cats() -> Vec<String> {
    vec!["body_measures".to_string()]
}

#[tokio::test]
async fn an_empty_period_writes_no_file() {
    let dir = tempfile::tempdir().unwrap();
    let pool = pool_with_one_weight(&dir).await;
    let out = dir.path().join("vide.csv");
    let rows = export_csv_inner(
        &pool,
        "2026-01-01T00:00:00.000Z",
        "2026-01-31T23:59:59.000Z",
        &cats(),
        &out,
    )
    .await
    .unwrap();
    assert_eq!(rows, 0);
    assert!(!out.exists());
    assert!(!dir.path().join("vide.csv.partial").exists());
}

#[tokio::test]
async fn the_last_day_is_included_and_decimals_use_a_dot() {
    let dir = tempfile::tempdir().unwrap();
    let pool = pool_with_one_weight(&dir).await;
    let out = dir.path().join("septembre.csv");
    let rows = export_csv_inner(
        &pool,
        "2026-09-01T00:00:00.000Z",
        "2026-09-30T23:59:59.000Z",
        &cats(),
        &out,
    )
    .await
    .unwrap();
    assert_eq!(rows, 1);
    let content = std::fs::read_to_string(&out).unwrap();
    let line = content
        .lines()
        .find(|l| l.starts_with("2026-09-30"))
        .expect("ligne du 30");
    assert_eq!(line.split(',').count(), 9, "{line}");
    assert!(line.contains("72.50"), "{line}");
}

async fn pool_with_every_category(dir: &tempfile::TempDir) -> sqlx::SqlitePool {
    let pool = pool_with_one_weight(dir).await;
    let inserts = [
        "INSERT INTO blood_pressure (measured_at, source, systolic, diastolic, heart_rate) VALUES ('2026-09-10T08:00:00Z', 'manual', 121, 79, 64)",
        "INSERT INTO sleep_records (date, source, start_at, end_at, duration_total, sleep_score) VALUES ('2026-09-11', 'manual', '2026-09-10T23:00:00Z', '2026-09-11T07:00:00Z', 28800, 82)",
        "INSERT INTO activity (date, source, steps, distance, calories_active) VALUES ('2026-09-12', 'manual', 9000, 6543.4, 321.6)",
        "INSERT INTO workouts (date, source, withings_id, category_label, duration, distance, calories, hr_average) VALUES ('2026-09-13', 'manual', 'w1', 'Course', 3600, 10000.4, 650.5, 140)",
        "INSERT INTO heart_records (timestamp, source, ecg_status, heart_rate) VALUES ('2026-09-14T10:00:00Z', 'manual', 1, 88)",
        "INSERT INTO measurement_types_legacy (slug, label, unit) VALUES ('tour_bras', 'Tour de bras', 'cm')",
        "INSERT INTO manual_measurements_legacy (measurement_type_id, measured_at, value) VALUES ((SELECT id FROM measurement_types_legacy WHERE slug = 'tour_bras'), '2026-09-15T09:00:00Z', 33.25)",
    ];
    for sql in inserts {
        sqlx::query(sql).execute(&pool).await.unwrap();
    }
    pool
}

#[tokio::test]
async fn every_category_is_written_with_its_header_and_formatted_rows() {
    let dir = tempfile::tempdir().unwrap();
    let pool = pool_with_every_category(&dir).await;
    let out = dir.path().join("tout.csv");
    let categories: Vec<String> = [
        "body_measures",
        "blood_pressure",
        "sleep",
        "activity",
        "measurements",
        "workouts",
        "heart",
    ]
    .iter()
    .map(|c| c.to_string())
    .collect();
    let rows = export_csv_inner(
        &pool,
        "2026-09-01T00:00:00.000Z",
        "2026-09-30T23:59:59.000Z",
        &categories,
        &out,
    )
    .await
    .unwrap();
    assert_eq!(rows, 7);
    let content = std::fs::read_to_string(&out).unwrap();
    for expected in [
        "# blood_pressure",
        "2026-09-10,121,79,64",
        "# sleep",
        "date,debut,fin,duree_totale_s,score_sommeil",
        "2026-09-11,2026-09-10T23:00:00Z,2026-09-11T07:00:00Z,28800,82",
        "# activity",
        "2026-09-12,9000,6543,322",
        "# measurements",
        "2026-09-15,Tour de bras,33.2,cm",
        "# workouts",
        "2026-09-13,Course,3600,10000,650,140",
        "# heart",
        "2026-09-14,AFib,88",
    ] {
        assert!(
            content.lines().any(|l| l == expected),
            "ligne absente : {expected}\n{content}"
        );
    }
}

#[tokio::test]
async fn free_text_that_looks_like_a_formula_is_neutralised_but_numbers_are_not() {
    let dir = tempfile::tempdir().unwrap();
    let pool = pool_with_one_weight(&dir).await;
    for sql in [
        "INSERT INTO measurement_types_legacy (slug, label, unit) VALUES ('piege', '=HYPERLINK(\"http://x\")', '@cmd')",
        "INSERT INTO manual_measurements_legacy (measurement_type_id, measured_at, value) VALUES ((SELECT id FROM measurement_types_legacy WHERE slug = 'piege'), '2026-09-15T09:00:00Z', -5.0)",
        "INSERT INTO workouts (date, source, withings_id, category_label, duration) VALUES ('2026-09-13', 'manual', 'w9', '+1+1', 60)",
        "INSERT INTO workouts (date, source, withings_id, category_label, duration) VALUES ('2026-09-14', 'manual', 'w10', 'Course', 60)",
    ] {
        sqlx::query(sql).execute(&pool).await.unwrap();
    }
    let out = dir.path().join("piege.csv");
    let categories: Vec<String> = ["measurements", "workouts"]
        .iter()
        .map(|c| c.to_string())
        .collect();

    export_csv_inner(
        &pool,
        "2026-09-01T00:00:00.000Z",
        "2026-09-30T23:59:59.000Z",
        &categories,
        &out,
    )
    .await
    .unwrap();

    let content = std::fs::read_to_string(&out).unwrap();
    assert!(
        content.contains("2026-09-15,\"'=HYPERLINK(\"\"http://x\"\")\",-5.0,'@cmd"),
        "{content}"
    );
    assert!(content.contains("2026-09-13,'+1+1,60"), "{content}");
    assert!(content.contains("2026-09-14,Course,60"), "{content}");
}
