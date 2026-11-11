use sqlx::SqlitePool;
use synk_lib::crypto::CryptoVault;

async fn make_pool() -> SqlitePool {
    let pool = crate::common::memory_pool().await;
    sqlx::query("INSERT OR IGNORE INTO source_credentials (source_id) VALUES ('withings')")
        .execute(&pool)
        .await
        .unwrap();
    pool
}

fn vault() -> CryptoVault {
    CryptoVault::from_key([0x33; 32])
}

#[tokio::test]
async fn set_credentials_writes_encrypted() {
    let pool = make_pool().await;
    let v = vault();

    crate::common::set_pair(&pool, &v, "my_client_id", "my_client_secret")
        .await
        .unwrap();

    let (cid, csec): (String, String) =
        sqlx::query_as("SELECT client_id, client_secret FROM withings_credentials WHERE id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(cid.starts_with("enc:v1:"), "client_id not encrypted: {cid}");
    assert!(
        csec.starts_with("enc:v1:"),
        "client_secret not encrypted: {csec}"
    );
}

#[tokio::test]
async fn get_credentials_returns_decrypted_round_trip() {
    let pool = make_pool().await;
    let v = vault();

    crate::common::set_pair(&pool, &v, "cid_value", "secret_value")
        .await
        .unwrap();

    let (cid, csec) = crate::common::get_pair(&pool, &v).await.unwrap();

    assert_eq!(cid.as_str(), "cid_value");
    assert_eq!(csec.as_str(), "secret_value");
}

#[tokio::test]
async fn get_credentials_empty_returns_empty_strings() {
    // First-run case: row id=1 exists but fields are empty
    let pool = make_pool().await;
    let v = vault();

    let (cid, csec) = crate::common::get_pair(&pool, &v).await.unwrap();
    assert_eq!(cid.as_str(), "");
    assert_eq!(csec.as_str(), "");
}

async fn make_pool_without_row() -> SqlitePool {
    let pool = crate::common::memory_pool().await;
    sqlx::query("DELETE FROM source_credentials")
        .execute(&pool)
        .await
        .unwrap();
    pool
}

#[tokio::test]
async fn set_credentials_creates_row_if_missing() {
    // First-run anomaly / partially restored DB:
    // row id=1 doesn't exist, set_pair must create it
    let pool = make_pool_without_row().await;
    let v = vault();

    crate::common::set_pair(&pool, &v, "missing_row_cid", "missing_row_secret")
        .await
        .unwrap();

    let (cid, csec) = crate::common::get_pair(&pool, &v).await.unwrap();

    assert_eq!(cid.as_str(), "missing_row_cid");
    assert_eq!(csec.as_str(), "missing_row_secret");
}
