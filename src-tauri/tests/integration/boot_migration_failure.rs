use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Connection, SqliteConnection};
use synk_lib::boot::{init_pool_with_retry, BootError};

#[tokio::test]
async fn a_failing_migration_is_reported_with_its_version() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("synk.db");
    let pool = synk_lib::db::init_pool(db_path.clone()).await.unwrap();
    pool.close().await;

    let options = SqliteConnectOptions::new().filename(&db_path);
    let mut conn = SqliteConnection::connect_with(&options).await.unwrap();
    sqlx::query("DELETE FROM schema_version WHERE version = 16")
        .execute(&mut conn)
        .await
        .unwrap();
    sqlx::query("CREATE TABLE clothing_sizes_new (x INTEGER)")
        .execute(&mut conn)
        .await
        .unwrap();
    conn.close().await.unwrap();

    let err = init_pool_with_retry(db_path).await.unwrap_err();

    match err {
        BootError::MigrationFailed { version, .. } => assert_eq!(version, 16),
        other => panic!("MigrationFailed attendu, obtenu {other:?}"),
    }
}
