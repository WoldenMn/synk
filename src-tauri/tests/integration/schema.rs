use synk_lib::db::{tolerated, LATEST_VERSION, MIGRATIONS};

const DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/migrations");

fn migration_files() -> Vec<String> {
    let mut files: Vec<String> = std::fs::read_dir(DIR)
        .expect("dossier des migrations")
        .map(|e| {
            e.expect("entree")
                .file_name()
                .into_string()
                .expect("nom utf-8")
        })
        .filter(|n| n.ends_with(".sql") && n != "001_init.sql")
        .collect();
    files.sort();
    files
}

#[test]
fn every_migration_file_is_registered_in_order() {
    let registered: Vec<&str> = MIGRATIONS.iter().map(|m| m.name).collect();
    assert_eq!(migration_files(), registered);
    for (i, m) in MIGRATIONS.iter().enumerate() {
        assert_eq!(m.version, i as i64 + 2, "{}", m.name);
        assert!(
            m.name.starts_with(&format!("{:03}_", m.version)),
            "{}",
            m.name
        );
    }
    assert_eq!(LATEST_VERSION, MIGRATIONS[MIGRATIONS.len() - 1].version);
}

#[tokio::test]
async fn a_new_database_ends_at_the_latest_version() {
    let pool = crate::common::memory_pool().await;
    let max: i64 = sqlx::query_scalar("SELECT MAX(version) FROM schema_version")
        .fetch_one(&pool)
        .await
        .expect("version");
    assert_eq!(max, LATEST_VERSION);
    assert_eq!(max, synk_lib::boot::EXPECTED_LATEST_VERSION);
}

#[test]
fn tolerance_is_frozen_after_version_16() {
    assert!(tolerated(16, "no such table: mood_logs"));
    assert!(tolerated(9, "duplicate column name: icon"));
    assert!(tolerated(2, "table x already exists"));
    assert!(!tolerated(17, "no such table: x"));
    assert!(!tolerated(17, "duplicate column name: x"));
    assert!(!tolerated(17, "table x already exists"));
    assert!(!tolerated(5, "syntax error"));
}

#[test]
fn no_semicolon_hides_in_a_comment_or_a_string() {
    for name in migration_files()
        .into_iter()
        .chain(["001_init.sql".to_string()])
    {
        let text = std::fs::read_to_string(format!("{DIR}/{name}")).expect("lecture");
        assert!(
            !text.to_uppercase().contains("CREATE TRIGGER"),
            "{name} : trigger, le decoupage sur ';' ne sait pas le lire"
        );
        let mut in_string = false;
        for line in text.lines() {
            let mut chars = line.chars().peekable();
            while let Some(c) = chars.next() {
                if !in_string && c == '-' && chars.peek() == Some(&'-') {
                    let comment: String = chars.collect();
                    assert!(
                        !comment.contains(';'),
                        "{name} : point-virgule en commentaire : {line}"
                    );
                    break;
                }
                if c == '\'' {
                    in_string = !in_string;
                } else if in_string && c == ';' {
                    panic!("{name} : point-virgule dans une chaine : {line}");
                }
            }
        }
    }
}

#[tokio::test]
async fn duplicate_indexes_are_gone() {
    let pool = crate::common::memory_pool().await;
    let names: Vec<String> =
        sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type = 'index'")
            .fetch_all(&pool)
            .await
            .expect("index");
    for dup in [
        "idx_legacy_body_measures_date",
        "idx_legacy_manual_type",
        "idx_legacy_manual_date",
        "idx_legacy_vitals_type",
        "idx_legacy_vitals_date",
        "idx_journal_date",
        "idx_mood_logs_date",
    ] {
        assert!(!names.iter().any(|n| n == dup), "{dup} existe encore");
    }
    for kept in [
        "idx_sleep_date",
        "idx_activity_date",
        "idx_workouts_date",
        "idx_heart_timestamp",
        "idx_body_measures_date",
        "idx_manual_measurements_date",
        "idx_manual_measurements_type",
        "idx_vitals_date",
        "idx_vitals_type",
    ] {
        assert!(names.iter().any(|n| n == kept), "{kept} a disparu");
    }
}

#[tokio::test]
async fn deleting_an_alert_deletes_its_events() {
    let pool = crate::common::memory_pool().await;
    let fk: i64 = sqlx::query_scalar("PRAGMA foreign_keys")
        .fetch_one(&pool)
        .await
        .expect("pragma");
    assert_eq!(fk, 1);
    let alert_id: i64 = sqlx::query_scalar(
        "INSERT INTO health_alerts (metric_key, condition, threshold, label, enabled) \
         VALUES ('weight', 'above', 90, 'Poids', 1) RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("alerte");
    for v in [91.0, 92.0] {
        sqlx::query(
            "INSERT INTO alert_events (alert_id, value, triggered_at) \
             VALUES (?, ?, '2026-10-03T08:00:00Z')",
        )
        .bind(alert_id)
        .bind(v)
        .execute(&pool)
        .await
        .expect("evenement");
    }
    sqlx::query("DELETE FROM health_alerts WHERE id = ?")
        .bind(alert_id)
        .execute(&pool)
        .await
        .expect("suppression");
    let left: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM alert_events")
        .fetch_one(&pool)
        .await
        .expect("compte");
    assert_eq!(left, 0);
}

#[tokio::test]
async fn deleting_a_medication_deletes_its_logs() {
    let pool = crate::common::memory_pool().await;
    let med_id: i64 =
        sqlx::query_scalar("INSERT INTO medications (name) VALUES ('Test') RETURNING id")
            .fetch_one(&pool)
            .await
            .expect("medicament");
    for day in ["2026-10-01", "2026-10-02"] {
        sqlx::query("INSERT INTO medication_logs (medication_id, taken_at) VALUES (?, ?)")
            .bind(med_id)
            .bind(day)
            .execute(&pool)
            .await
            .expect("prise");
    }
    sqlx::query("DELETE FROM medications WHERE id = ?")
        .bind(med_id)
        .execute(&pool)
        .await
        .expect("suppression");
    let left: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM medication_logs")
        .fetch_one(&pool)
        .await
        .expect("compte");
    assert_eq!(left, 0);
}

#[tokio::test]
async fn migration_017_keeps_existing_rows() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("base");
    for sql in [
        "CREATE TABLE schema_version (version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL DEFAULT (datetime('now')))",
        "CREATE TABLE health_alerts (id INTEGER PRIMARY KEY AUTOINCREMENT)",
        "CREATE TABLE medications (id INTEGER PRIMARY KEY AUTOINCREMENT)",
        "CREATE TABLE alert_events (id INTEGER PRIMARY KEY AUTOINCREMENT, alert_id INTEGER REFERENCES health_alerts(id), value REAL NOT NULL, triggered_at TEXT NOT NULL, seen INTEGER DEFAULT 0)",
        "CREATE INDEX idx_alert_events_alert ON alert_events(alert_id)",
        "CREATE INDEX idx_alert_events_date ON alert_events(triggered_at)",
        "CREATE TABLE medication_logs (id INTEGER PRIMARY KEY AUTOINCREMENT, medication_id INTEGER REFERENCES medications(id), taken_at TEXT NOT NULL, skipped INTEGER DEFAULT 0, notes TEXT, created_at TEXT NOT NULL DEFAULT (datetime('now')))",
        "CREATE INDEX idx_medication_logs_med ON medication_logs(medication_id)",
        "CREATE INDEX idx_medication_logs_date ON medication_logs(taken_at)",
        "INSERT INTO health_alerts (id) VALUES (1)",
        "INSERT INTO medications (id) VALUES (1)",
        "INSERT INTO alert_events (id, alert_id, value, triggered_at, seen) VALUES (7, 1, 91.5, '2026-10-01T08:00:00Z', 1)",
        "INSERT INTO medication_logs (id, medication_id, taken_at, skipped, notes) VALUES (9, 1, '2026-10-02', 1, 'note')",
    ] {
        sqlx::query(sql).execute(&pool).await.expect(sql);
    }
    let m17 = MIGRATIONS
        .iter()
        .find(|m| m.version == 17)
        .expect("017 enregistree");
    synk_lib::db::run_versioned_migration(&pool, 17, m17.sql)
        .await
        .expect("017");

    let event: (i64, f64, String, i64) =
        sqlx::query_as("SELECT alert_id, value, triggered_at, seen FROM alert_events WHERE id = 7")
            .fetch_one(&pool)
            .await
            .expect("evenement");
    assert_eq!(event, (1, 91.5, "2026-10-01T08:00:00Z".to_string(), 1));
    let log: (i64, String, i64, String) = sqlx::query_as(
        "SELECT medication_id, taken_at, skipped, notes FROM medication_logs WHERE id = 9",
    )
    .fetch_one(&pool)
    .await
    .expect("prise");
    assert_eq!(log, (1, "2026-10-02".to_string(), 1, "note".to_string()));
    for table in ["alert_events", "medication_logs"] {
        let ddl: String = sqlx::query_scalar("SELECT sql FROM sqlite_master WHERE name = ?")
            .bind(table)
            .fetch_one(&pool)
            .await
            .expect("ddl");
        assert!(ddl.contains("ON DELETE CASCADE"), "{ddl}");
    }
    let indexes: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type = 'index' AND name LIKE 'idx_%' ORDER BY name",
    )
    .fetch_all(&pool)
    .await
    .expect("index");
    assert_eq!(
        indexes,
        [
            "idx_alert_events_alert",
            "idx_alert_events_date",
            "idx_medication_logs_date",
            "idx_medication_logs_med"
        ]
    );
    let violations: Vec<(String,)> = sqlx::query_as("PRAGMA foreign_key_check")
        .fetch_all(&pool)
        .await
        .expect("controle");
    assert!(violations.is_empty());
}

#[tokio::test]
async fn a_fresh_base_has_no_dead_v2_table() {
    let pool = crate::common::memory_pool().await;
    for table in ["derived_metrics", "metric_targets"] {
        let found: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sqlite_master WHERE name = ?1")
            .bind(table)
            .fetch_one(&pool)
            .await
            .expect("catalogue");
        assert_eq!(found, 0, "{table} doit etre retiree par la 018");
    }
    let index_left: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name LIKE 'idx_derived_metrics%'",
    )
    .fetch_one(&pool)
    .await
    .expect("index");
    assert_eq!(index_left, 0);
}

#[tokio::test]
async fn migration_018_removes_only_the_two_dead_tables() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("base");
    for sql in [
        "CREATE TABLE schema_version (version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL DEFAULT (datetime('now')))",
        "CREATE TABLE metric_definitions (id INTEGER PRIMARY KEY, slug TEXT)",
        "CREATE TABLE metrics (id INTEGER PRIMARY KEY, definition_id INTEGER REFERENCES metric_definitions(id) ON DELETE CASCADE, measured_at TEXT, value REAL)",
        "CREATE TABLE goals (id INTEGER PRIMARY KEY, metric_key TEXT, target_value REAL)",
        "CREATE TABLE metric_targets (id INTEGER PRIMARY KEY, definition_id INTEGER UNIQUE REFERENCES metric_definitions(id) ON DELETE CASCADE, target_value REAL)",
        "CREATE TABLE derived_metrics (id INTEGER PRIMARY KEY, definition_id INTEGER REFERENCES metric_definitions(id) ON DELETE CASCADE, measured_at TEXT, value REAL)",
        "CREATE INDEX idx_derived_metrics_definition_date ON derived_metrics(definition_id, measured_at)",
        "INSERT INTO metric_definitions (id, slug) VALUES (1, 'weight')",
        "INSERT INTO metrics (id, definition_id, measured_at, value) VALUES (1, 1, '2026-10-01T08:00:00Z', 71.5)",
        "INSERT INTO goals (id, metric_key, target_value) VALUES (1, 'weight', 70.0)",
        "INSERT INTO metric_targets (id, definition_id, target_value) VALUES (1, 1, 70.0)",
    ] {
        sqlx::query(sql).execute(&pool).await.expect(sql);
    }
    let m18 = MIGRATIONS
        .iter()
        .find(|m| m.version == 18)
        .expect("018 enregistree");

    synk_lib::db::run_versioned_migration(&pool, 18, m18.sql)
        .await
        .expect("018");

    let names: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE name IN ('derived_metrics', 'metric_targets', 'idx_derived_metrics_definition_date')",
    )
    .fetch_all(&pool)
    .await
    .expect("catalogue");
    assert!(names.is_empty(), "restent : {names:?}");
    for (table, expected) in [("metrics", 1i64), ("goals", 1), ("metric_definitions", 1)] {
        let n: i64 =
            sqlx::query_scalar(sqlx::AssertSqlSafe(format!("SELECT COUNT(*) FROM {table}")))
                .fetch_one(&pool)
                .await
                .expect("compte");
        assert_eq!(n, expected, "{table} ne perd aucune ligne");
    }
}
