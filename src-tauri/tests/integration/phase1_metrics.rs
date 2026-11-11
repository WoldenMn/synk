use sqlx::SqlitePool;
use synk_lib::db::{bootstrap_schema, run_post_ready_seed};

#[tokio::test]
async fn test_fresh_db_creates_the_two_metric_tables() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    bootstrap_schema(&pool).await.unwrap();

    let tables: Vec<(String,)> = sqlx::query_as(
        "SELECT name FROM sqlite_master WHERE type='table'
         AND name IN ('metric_definitions','metrics')
         ORDER BY name",
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    assert_eq!(
        tables.len(),
        2,
        "metric_definitions et metrics restent, les deux tables mortes sont retirees par la 018"
    );

    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM metric_definitions WHERE is_builtin = 1")
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(count.0, 52, "Seed attendu : 52 metric_definitions built-in (36 V2.6 + 5 V2.7 vital_* + 1 V13 temperature + 3 Oura + 2 Polar + 3 Whoop + 2 Apple Sante declarees par le registre des sources, R35 a R39)");
}

#[tokio::test]
async fn test_migration_preserves_data_via_legacy_views() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    // Hypothese : pas de `;` dans les strings SQL de 001_init.sql (sinon le
    // split naif divergerait du parser sqlx utilise en prod).
    let init_sql = include_str!("../../migrations/001_init.sql");
    for statement in init_sql.split(';') {
        let trimmed = statement.trim();
        if !trimmed.is_empty() {
            let _ = sqlx::query(trimmed).execute(&pool).await;
        }
    }

    sqlx::query(
        "INSERT INTO body_measures (measured_at, source, weight, fat_ratio)
         VALUES ('2026-04-01T10:00:00', 'withings', 75.2, 22.5)",
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO measurement_types (slug, label, unit, category, is_builtin, sort_order)
         VALUES ('test_custom', 'Test Custom', 'cm', 'circumference', 0, 99)",
    )
    .execute(&pool)
    .await
    .unwrap();

    bootstrap_schema(&pool).await.unwrap();
    run_post_ready_seed(&pool).await;

    let legacy_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM body_measures_legacy")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        legacy_count.0, 1,
        "Donnee preservee dans body_measures_legacy"
    );

    let view_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM body_measures")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        view_count.0, 1,
        "Vue body_measures retourne les donnees legacy"
    );

    let custom_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM metric_definitions WHERE category = 'custom' AND slug = 'test_custom'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(custom_count.0, 1, "Custom type migre en metric_definitions");
}

// Regression test pour DB#1 : apres bootstrap_schema, body_measures devient une VUE.
// SQLite refuse INSERT/UPDATE/DELETE sur VUE sans trigger INSTEAD OF.
// Le fix consiste a pointer le code vers `body_measures_legacy` directement.
#[tokio::test]
async fn test_inserts_into_legacy_tables_after_bootstrap() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    bootstrap_schema(&pool).await.unwrap();

    // INSERT direct dans la table legacy doit marcher (c'est une vraie table apres rename).
    sqlx::query(
        "INSERT INTO body_measures_legacy (measured_at, source, weight)
         VALUES ('2026-04-26T08:00:00Z', 'manual', 72.5)",
    )
    .execute(&pool)
    .await
    .expect("INSERT INTO body_measures_legacy doit reussir apres bootstrap");

    // SELECT via la vue doit retourner la donnee inseree.
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM body_measures WHERE weight = 72.5")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count.0, 1, "La vue retrocompat reflete les donnees legacy");

    // INSERT direct dans la VUE doit echouer (preuve que sans trigger INSTEAD OF c'est casse).
    let view_insert = sqlx::query(
        "INSERT INTO body_measures (measured_at, source, weight) VALUES ('2026-04-27T08:00:00Z', 'manual', 73.0)",
    )
    .execute(&pool)
    .await;
    assert!(
        view_insert.is_err(),
        "INSERT INTO la VUE body_measures doit echouer (regression DB#1)"
    );
}

#[tokio::test]
async fn test_read_metric_latest_returns_none_for_unknown_slug() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    bootstrap_schema(&pool).await.unwrap();

    let result = synk_lib::metric_repository::read_metric_latest(&pool, "nonexistent_slug")
        .await
        .unwrap();

    assert!(result.is_none(), "Slug inconnu doit retourner None");
}

#[tokio::test]
async fn test_read_metric_latest_reads_from_metrics_table() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    bootstrap_schema(&pool).await.unwrap();

    let (def_id,): (i64,) = sqlx::query_as("SELECT id FROM metric_definitions WHERE slug = 'ffmi'")
        .fetch_one(&pool)
        .await
        .unwrap();

    for (date, value) in &[
        ("2026-01-01T08:00:00Z", 70.0),
        ("2026-03-01T08:00:00Z", 72.5),
        ("2026-02-15T08:00:00Z", 71.2),
    ] {
        sqlx::query(
            "INSERT INTO metrics (definition_id, measured_at, value, source) \
             VALUES (?, ?, ?, 'manual')",
        )
        .bind(def_id)
        .bind(date)
        .bind(*value)
        .execute(&pool)
        .await
        .unwrap();
    }

    let result = synk_lib::metric_repository::read_metric_latest(&pool, "ffmi")
        .await
        .unwrap()
        .expect("Doit retourner une valeur");

    assert_eq!(
        result.measured_at, "2026-03-01T08:00:00Z",
        "Doit retourner le plus recent"
    );
    assert_eq!(result.value, 72.5);
}

#[tokio::test]
async fn test_read_metric_latest_reads_from_legacy_when_no_metrics_entry() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    bootstrap_schema(&pool).await.unwrap();

    for (date, weight) in &[
        ("2026-01-10T09:00:00Z", 75.0),
        ("2026-02-20T09:00:00Z", 76.5),
    ] {
        sqlx::query(
            "INSERT INTO body_measures_legacy (measured_at, source, weight) \
             VALUES (?, 'manual', ?)",
        )
        .bind(date)
        .bind(*weight)
        .execute(&pool)
        .await
        .unwrap();
    }

    let result = synk_lib::metric_repository::read_metric_latest(&pool, "weight")
        .await
        .unwrap()
        .expect("Doit retourner la valeur legacy");

    assert_eq!(
        result.measured_at, "2026-02-20T09:00:00Z",
        "Doit retourner le plus recent legacy"
    );
    assert_eq!(result.value, 76.5);
}

#[tokio::test]
async fn test_read_metric_latest_ignores_a_stale_metrics_row_for_a_legacy_metric() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    bootstrap_schema(&pool).await.unwrap();

    let (def_id,): (i64,) =
        sqlx::query_as("SELECT id FROM metric_definitions WHERE slug = 'weight'")
            .fetch_one(&pool)
            .await
            .unwrap();

    sqlx::query("INSERT INTO metrics (definition_id, measured_at, value, source) VALUES (?, '2026-03-01T08:00:00Z', 70.0, 'manual')")
        .bind(def_id)
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO body_measures_legacy (measured_at, source, weight) VALUES ('2026-02-01T08:00:00Z', 'manual', 75.0)")
        .execute(&pool)
        .await
        .unwrap();

    let result = synk_lib::metric_repository::read_metric_latest(&pool, "weight")
        .await
        .unwrap()
        .expect("Doit retourner une valeur");

    assert_eq!(result.measured_at, "2026-02-01T08:00:00Z");
    assert_eq!(result.value, 75.0);
}

#[tokio::test]
async fn test_migration_009_preserves_existing_rows_and_check_constraint() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    bootstrap_schema(&pool).await.unwrap();

    // 52 entries : 36 V2.6 + 5 V2.7 vital_* (migration 009) + 1 V13 temperature (migration 013) + 3 Oura + 2 Polar + 3 Whoop + 2 Apple Sante (seed Rust des sources, R35 a R39).
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM metric_definitions")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        count.0, 52,
        "36 + 5 (009) + 1 temperature (013) + 3 Oura (R35) + 2 Polar (R36) + 3 Whoop (R37) + 2 Apple Sante (R39) = 52"
    );

    // Verifier les 5 nouveaux slugs presents avec bonnes categories.
    let bp_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM metric_definitions WHERE category = 'vital_bp'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        bp_count.0, 3,
        "vital_bp doit contenir systolic/diastolic/heart_rate"
    );

    let sleep_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM metric_definitions WHERE category = 'vital_sleep'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(sleep_count.0, 1, "vital_sleep doit contenir sleep_score");

    let activity_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM metric_definitions WHERE category = 'vital_activity'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(activity_count.0, 1, "vital_activity doit contenir steps");

    // CHECK constraint accepte les 3 nouvelles categories.
    let insert_bp = sqlx::query(
        "INSERT INTO metric_definitions (slug, label, unit, category, is_builtin, sort_order, decimals)
         VALUES ('test_bp', 'Test', 'mmHg', 'vital_bp', 0, 999, 0)"
    ).execute(&pool).await;
    assert!(insert_bp.is_ok(), "Categorie vital_bp doit etre acceptee");

    // CHECK constraint rejette une categorie inexistante.
    let insert_bogus = sqlx::query(
        "INSERT INTO metric_definitions (slug, label, unit, category, is_builtin, sort_order, decimals)
         VALUES ('test_bogus', 'Test', 'x', 'bogus_category', 0, 998, 0)"
    ).execute(&pool).await;
    assert!(
        insert_bogus.is_err(),
        "Categorie inexistante doit etre rejetee"
    );
}

#[tokio::test]
async fn test_read_metric_latest_returns_systolic_from_blood_pressure() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    bootstrap_schema(&pool).await.unwrap();

    sqlx::query(
        "INSERT INTO blood_pressure (measured_at, source, systolic, diastolic) \
         VALUES ('2026-05-15T08:00:00Z', 'manual', 140, 90)",
    )
    .execute(&pool)
    .await
    .unwrap();

    let result = synk_lib::metric_repository::read_metric_latest(&pool, "systolic")
        .await
        .unwrap()
        .expect("Doit retourner une valeur");
    assert_eq!(result.value, 140.0);
}

#[tokio::test]
async fn test_read_metric_latest_returns_sleep_score_from_sleep_records() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    bootstrap_schema(&pool).await.unwrap();

    sqlx::query(
        "INSERT INTO sleep_records (date, source, sleep_score) \
         VALUES ('2026-05-15', 'manual', 75)",
    )
    .execute(&pool)
    .await
    .unwrap();

    let result = synk_lib::metric_repository::read_metric_latest(&pool, "sleep_score")
        .await
        .unwrap()
        .expect("Doit retourner une valeur");
    assert_eq!(result.value, 75.0);
}

#[tokio::test]
async fn test_read_metric_latest_returns_steps_from_activity() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    bootstrap_schema(&pool).await.unwrap();

    sqlx::query(
        "INSERT INTO activity (date, source, steps) \
         VALUES ('2026-05-15', 'manual', 8000)",
    )
    .execute(&pool)
    .await
    .unwrap();

    let result = synk_lib::metric_repository::read_metric_latest(&pool, "steps")
        .await
        .unwrap()
        .expect("Doit retourner une valeur");
    assert_eq!(result.value, 8000.0);
}
