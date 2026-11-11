use sqlx::SqlitePool;
use synk_lib::crypto::{migrate_plaintext_secrets, CryptoVault, MigrationReport};

async fn make_pool() -> SqlitePool {
    let pool = crate::common::memory_pool().await;
    sqlx::query("DELETE FROM source_credentials")
        .execute(&pool)
        .await
        .unwrap();
    pool
}

fn vault() -> CryptoVault {
    CryptoVault::from_key([0x55u8; 32])
}

#[tokio::test]
async fn migrates_plaintext_auth_row() {
    let pool = make_pool().await;
    sqlx::query("INSERT INTO source_auth (source_id, access_token, refresh_token, expires_at) VALUES ('withings', 'plain_at', 'plain_rt', 0)")
        .execute(&pool).await.unwrap();

    let v = vault();
    let report = migrate_plaintext_secrets(&pool, &v).await.unwrap();
    assert_eq!(report.auth_rows_encrypted, 1);

    let (at, rt): (String, String) =
        sqlx::query_as("SELECT access_token, refresh_token FROM withings_auth WHERE id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(
        at.starts_with("enc:v1:"),
        "access_token not encrypted: {at}"
    );
    assert!(rt.starts_with("enc:v1:"));
    assert_eq!(v.open_str(&at).unwrap().as_str(), "plain_at");
    assert_eq!(v.open_str(&rt).unwrap().as_str(), "plain_rt");
}

#[tokio::test]
async fn migrates_plaintext_credentials_row() {
    let pool = make_pool().await;
    sqlx::query("INSERT INTO source_credentials (source_id, client_id, client_secret) VALUES ('withings', 'cid', 'csecret')")
        .execute(&pool).await.unwrap();

    let v = vault();
    let report = migrate_plaintext_secrets(&pool, &v).await.unwrap();
    assert_eq!(report.credentials_rows_encrypted, 1);

    let (cid, csec): (String, String) =
        sqlx::query_as("SELECT client_id, client_secret FROM withings_credentials WHERE id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(cid.starts_with("enc:v1:"));
    assert!(csec.starts_with("enc:v1:"));
    assert_eq!(v.open_str(&cid).unwrap().as_str(), "cid");
    assert_eq!(v.open_str(&csec).unwrap().as_str(), "csecret");
}

#[tokio::test]
async fn migration_is_idempotent() {
    let pool = make_pool().await;
    sqlx::query("INSERT INTO source_auth (source_id, access_token, refresh_token, expires_at) VALUES ('withings', 'a', 'b', 0)")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO source_credentials (source_id, client_id, client_secret) VALUES ('withings', 'c', 'd')",
    )
    .execute(&pool)
    .await
    .unwrap();

    let v = vault();
    let report1 = migrate_plaintext_secrets(&pool, &v).await.unwrap();
    assert_eq!(
        report1.auth_rows_encrypted + report1.credentials_rows_encrypted,
        2
    );

    let report2 = migrate_plaintext_secrets(&pool, &v).await.unwrap();
    assert_eq!(report2, MigrationReport::default());
}

#[tokio::test]
async fn migration_noop_on_empty_tables() {
    let pool = make_pool().await;
    let v = vault();
    let report = migrate_plaintext_secrets(&pool, &v).await.unwrap();
    assert_eq!(report, MigrationReport::default());
}

#[tokio::test]
async fn migration_mixed_row_only_encrypts_plaintext() {
    let pool = make_pool().await;
    let v = vault();
    let already_encrypted = v.seal_str("already").unwrap();
    sqlx::query(
        "INSERT INTO source_auth (source_id, access_token, refresh_token, expires_at) VALUES ('withings', ?1, 'still_plain', 0)",
    )
    .bind(&already_encrypted)
    .execute(&pool)
    .await
    .unwrap();

    let report = migrate_plaintext_secrets(&pool, &v).await.unwrap();
    assert_eq!(report.auth_rows_encrypted, 1);

    let (at, rt): (String, String) =
        sqlx::query_as("SELECT access_token, refresh_token FROM withings_auth WHERE id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(at, already_encrypted); // unchanged
    assert!(rt.starts_with("enc:v1:")); // encrypted
    assert_eq!(v.open_str(&rt).unwrap().as_str(), "still_plain");
}
