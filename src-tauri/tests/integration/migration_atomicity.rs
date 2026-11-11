use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use synk_lib::db::{bootstrap_schema, migration_001_replay_is_benign, run_versioned_migration};

async fn pool_with_version_table() -> SqlitePool {
    let pool = SqlitePool::connect(":memory:").await.unwrap();
    sqlx::query(
        "CREATE TABLE schema_version (version INTEGER PRIMARY KEY, \
         applied_at TEXT NOT NULL DEFAULT (datetime('now')))",
    )
    .execute(&pool)
    .await
    .unwrap();
    pool
}

async fn table_exists(pool: &SqlitePool, name: &str) -> bool {
    let row: Option<String> =
        sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type='table' AND name = ?1")
            .bind(name)
            .fetch_optional(pool)
            .await
            .unwrap();
    row.is_some()
}

async fn version_applied(pool: &SqlitePool, version: i64) -> bool {
    let row: Option<i64> =
        sqlx::query_scalar("SELECT version FROM schema_version WHERE version = ?1")
            .bind(version)
            .fetch_optional(pool)
            .await
            .unwrap();
    row.is_some()
}

#[tokio::test]
async fn applies_a_valid_migration() {
    let pool = pool_with_version_table().await;
    let sql = "CREATE TABLE a (id INTEGER PRIMARY KEY); \
               CREATE TABLE b (id INTEGER PRIMARY KEY)";
    run_versioned_migration(&pool, 100, sql).await.unwrap();

    assert!(table_exists(&pool, "a").await);
    assert!(table_exists(&pool, "b").await);
    assert!(version_applied(&pool, 100).await);
}

#[tokio::test]
async fn rolls_back_everything_when_a_statement_fails() {
    let pool = pool_with_version_table().await;
    // La deuxieme instruction est invalide : rien de la premiere ne doit rester.
    let sql = "CREATE TABLE a (id INTEGER PRIMARY KEY); \
               CREATE TABLE b (this is not valid sql)";
    let result = run_versioned_migration(&pool, 101, sql).await;

    assert!(result.is_err(), "une migration invalide doit echouer");
    assert!(
        !table_exists(&pool, "a").await,
        "la premiere table doit avoir ete annulee"
    );
    assert!(
        !version_applied(&pool, 101).await,
        "une migration echouee ne doit pas etre marquee appliquee"
    );
}

#[tokio::test]
async fn a_pragma_outside_the_transaction_takes_effect() {
    let pool = pool_with_version_table().await;
    sqlx::query("PRAGMA foreign_keys=ON")
        .execute(&pool)
        .await
        .unwrap();

    // Reconstruction de table facon migration 009/016 : le PRAGMA doit
    // reellement desactiver les FK, sinon le DROP echouerait.
    let sql = "PRAGMA foreign_keys=OFF; \
               CREATE TABLE parent (id INTEGER PRIMARY KEY); \
               CREATE TABLE child (id INTEGER PRIMARY KEY, p INTEGER REFERENCES parent(id)); \
               CREATE TABLE parent_new (id INTEGER PRIMARY KEY); \
               DROP TABLE parent; \
               ALTER TABLE parent_new RENAME TO parent; \
               PRAGMA foreign_keys=ON";
    run_versioned_migration(&pool, 102, sql).await.unwrap();

    assert!(table_exists(&pool, "parent").await);
    assert!(version_applied(&pool, 102).await);
}

#[tokio::test]
async fn an_already_applied_migration_is_skipped() {
    let pool = pool_with_version_table().await;
    run_versioned_migration(&pool, 103, "CREATE TABLE once (id INTEGER)")
        .await
        .unwrap();
    // Rejouer la meme version avec un SQL qui echouerait doit etre un no-op.
    run_versioned_migration(&pool, 103, "CREATE TABLE once (id INTEGER)")
        .await
        .unwrap();
}

// Contenu reel embarque, pas du SQL synthetique : c'est precisement l'ecart
// entre les deux qui avait laisse passer une regression. Le fichier fait
// preceder son PRAGMA d'un bloc de commentaires sans ';', donc le premier
// fragment ne commence pas par "PRAGMA".
const MIGRATION_009_REAL: &str =
    include_str!("../../migrations/009_metric_definitions_extension.sql");

async fn pool_with_metrics_data() -> SqlitePool {
    let pool = SqlitePool::connect(":memory:").await.unwrap();
    sqlx::query("PRAGMA foreign_keys=ON")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "CREATE TABLE schema_version (version INTEGER PRIMARY KEY, \
         applied_at TEXT NOT NULL DEFAULT (datetime('now')))",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "CREATE TABLE metric_definitions (\
            id INTEGER PRIMARY KEY AUTOINCREMENT, slug TEXT NOT NULL UNIQUE, \
            label TEXT NOT NULL, unit TEXT NOT NULL, \
            category TEXT NOT NULL CHECK(category IN ('body_composition','circumference','body_scan','derived','custom')), \
            withings_type INTEGER, is_builtin INTEGER NOT NULL DEFAULT 0, \
            sort_order INTEGER NOT NULL DEFAULT 100, decimals INTEGER NOT NULL DEFAULT 2, \
            min_value REAL, max_value REAL, \
            created_at TEXT NOT NULL DEFAULT (datetime('now')))",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "CREATE TABLE metrics (\
            id INTEGER PRIMARY KEY AUTOINCREMENT, \
            definition_id INTEGER NOT NULL REFERENCES metric_definitions(id) ON DELETE CASCADE, \
            measured_at TEXT NOT NULL, value REAL NOT NULL)",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO metric_definitions (id, slug, label, unit, category) \
         VALUES (1, 'weight', 'Poids', 'kg', 'body_composition')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO metrics (definition_id, measured_at, value) VALUES (1, '2026-01-01', 82.0)",
    )
    .execute(&pool)
    .await
    .unwrap();
    pool
}

#[tokio::test]
async fn migration_009_does_not_cascade_delete_existing_metrics() {
    let pool = pool_with_metrics_data().await;

    let before: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM metrics")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(before, 1, "donnee de depart");

    run_versioned_migration(&pool, 9, MIGRATION_009_REAL)
        .await
        .unwrap();

    let after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM metrics")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        after, 1,
        "les mesures doivent survivre a la reconstruction de metric_definitions"
    );
}

// Verrouille une affirmation portee par CLAUDE.md et par le commentaire de
// run_versioned_migration : contrairement a PostgreSQL, SQLite n'avorte pas
// une transaction apres une erreur. Les instructions suivantes s'executent
// et le COMMIT persiste l'ensemble. Sans ce test, l'affirmation restait une
// promesse.
#[tokio::test]
async fn an_ignored_error_does_not_poison_the_transaction() {
    let pool = pool_with_version_table().await;
    sqlx::query("CREATE TABLE deja (id INTEGER PRIMARY KEY)")
        .execute(&pool)
        .await
        .unwrap();

    // La 1re instruction echoue avec "already exists" (erreur ignorable),
    // la 2e doit malgre tout s'appliquer et survivre au COMMIT.
    let sql = "CREATE TABLE deja (id INTEGER PRIMARY KEY); \
               CREATE TABLE apres (id INTEGER PRIMARY KEY)";
    run_versioned_migration(&pool, 16, sql).await.unwrap();

    assert!(
        table_exists(&pool, "apres").await,
        "l'instruction suivant une erreur ignoree doit etre appliquee"
    );
    assert!(version_applied(&pool, 16).await);
}

#[tokio::test]
async fn a_migration_after_16_does_not_tolerate_an_existing_object() {
    let pool = pool_with_version_table().await;
    sqlx::query("CREATE TABLE deja (id INTEGER PRIMARY KEY)")
        .execute(&pool)
        .await
        .unwrap();

    let sql = "CREATE TABLE deja (id INTEGER PRIMARY KEY); \
               CREATE TABLE apres (id INTEGER PRIMARY KEY)";
    let error = run_versioned_migration(&pool, 110, sql).await.unwrap_err();

    assert_eq!(error.version, 110);
    assert!(!table_exists(&pool, "apres").await);
    assert!(!version_applied(&pool, 110).await);
}

// MIGRATION_001 est rejouee a chaque demarrage sur une base deja a jour. Ses
// echecs doivent tous etre ceux, attendus, d'objets deja presents ou devenus des
// vues : un echec non reconnu part en avertissement dans le journal de
// l'utilisateur a chaque lancement (5 par demarrage, releve du 2026-10-02).
#[tokio::test]
async fn migration_001_replay_on_an_up_to_date_base_only_fails_benignly() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    bootstrap_schema(&pool).await.unwrap();

    let mut failures = Vec::new();
    for statement in include_str!("../../migrations/001_init.sql")
        .split(';')
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        if let Err(e) = sqlx::query(statement).execute(&pool).await {
            failures.push(e.to_string());
        }
    }

    assert!(
        !failures.is_empty(),
        "le rejeu ne produit plus aucun echec : ce test ne verifie plus rien"
    );
    let unexpected: Vec<&String> = failures
        .iter()
        .filter(|msg| !migration_001_replay_is_benign(msg))
        .collect();
    assert!(
        unexpected.is_empty(),
        "echecs du rejeu classes comme anormaux : {unexpected:?}"
    );
}

async fn pool_with_orphan_prone_tables() -> SqlitePool {
    let pool = pool_with_version_table().await;
    sqlx::query("CREATE TABLE parent (id INTEGER PRIMARY KEY)")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("CREATE TABLE child (id INTEGER PRIMARY KEY, p INTEGER REFERENCES parent(id))")
        .execute(&pool)
        .await
        .unwrap();
    pool
}

#[tokio::test]
async fn a_strict_migration_that_adds_a_foreign_key_violation_is_rolled_back() {
    let pool = pool_with_orphan_prone_tables().await;

    let sql = "PRAGMA foreign_keys=OFF; \
               INSERT INTO child (id, p) VALUES (1, 99); \
               PRAGMA foreign_keys=ON";
    let error = run_versioned_migration(&pool, 120, sql).await.unwrap_err();

    assert_eq!(error.version, 120);
    assert!(error.to_string().contains("foreign"), "{error}");
    let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM child")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(rows, 0);
    assert!(!version_applied(&pool, 120).await);
}

#[tokio::test]
async fn a_strict_migration_does_not_answer_for_violations_that_were_already_there() {
    let pool = pool_with_orphan_prone_tables().await;
    run_versioned_migration(
        &pool,
        5,
        "PRAGMA foreign_keys=OFF; INSERT INTO child (id, p) VALUES (1, 99); PRAGMA foreign_keys=ON",
    )
    .await
    .unwrap();

    run_versioned_migration(&pool, 121, "CREATE TABLE autre (id INTEGER)")
        .await
        .unwrap();

    assert!(version_applied(&pool, 121).await);
}
