use sqlx::SqlitePool;
use synk_lib::crypto::CryptoVault;
use synk_lib::sources::withings::types::TokenBody;

async fn make_pool() -> SqlitePool {
    crate::common::memory_pool().await
}

fn vault() -> CryptoVault {
    CryptoVault::from_key([0x7a; 32])
}

fn fake_tokens() -> TokenBody {
    TokenBody {
        access_token: "AT-12345".to_string(),
        refresh_token: "RT-67890".to_string(),
        expires_in: 3600,
        token_type: "Bearer".to_string(),
        userid: serde_json::Value::String("USER-1".to_string()),
    }
}

#[tokio::test]
async fn save_tokens_writes_encrypted_columns() {
    let pool = make_pool().await;
    let v = vault();
    let tokens = fake_tokens();

    synk_lib::sources::withings::client::save_tokens(&pool, &tokens, &v)
        .await
        .unwrap();

    let (at, rt): (String, String) =
        sqlx::query_as("SELECT access_token, refresh_token FROM withings_auth WHERE id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();

    assert!(
        at.starts_with("enc:v1:"),
        "access_token not encrypted: {at}"
    );
    assert!(
        rt.starts_with("enc:v1:"),
        "refresh_token not encrypted: {rt}"
    );
    assert_eq!(v.open_str(&at).unwrap().as_str(), "AT-12345");
    assert_eq!(v.open_str(&rt).unwrap().as_str(), "RT-67890");
}

#[tokio::test]
async fn save_tokens_overwrites_existing_row() {
    let pool = make_pool().await;
    let v = vault();
    let tokens1 = fake_tokens();
    let mut tokens2 = fake_tokens();
    tokens2.access_token = "AT-NEW".to_string();
    tokens2.refresh_token = "RT-NEW".to_string();

    synk_lib::sources::withings::client::save_tokens(&pool, &tokens1, &v)
        .await
        .unwrap();
    synk_lib::sources::withings::client::save_tokens(&pool, &tokens2, &v)
        .await
        .unwrap();

    let (at, rt): (String, String) =
        sqlx::query_as("SELECT access_token, refresh_token FROM withings_auth WHERE id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(v.open_str(&at).unwrap().as_str(), "AT-NEW");
    assert_eq!(v.open_str(&rt).unwrap().as_str(), "RT-NEW");
}

#[tokio::test]
async fn raw_db_read_returns_encrypted_blob() {
    // Defensive test: confirms the DB stores ciphertext (not plaintext)
    // even when reading raw SQL without going through vault.open_str()
    let pool = make_pool().await;
    let v = vault();
    synk_lib::sources::withings::client::save_tokens(&pool, &fake_tokens(), &v)
        .await
        .unwrap();

    let (at, rt): (String, String) =
        sqlx::query_as("SELECT access_token, refresh_token FROM withings_auth WHERE id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();

    // Plaintext values from fake_tokens() must NOT appear in raw DB content
    assert!(
        !at.contains("AT-12345"),
        "access_token leaked plaintext: {at}"
    );
    assert!(
        !rt.contains("RT-67890"),
        "refresh_token leaked plaintext: {rt}"
    );
    // Must have ciphertext prefix
    assert!(at.starts_with("enc:v1:"), "access_token not prefixed: {at}");
    assert!(
        rt.starts_with("enc:v1:"),
        "refresh_token not prefixed: {rt}"
    );
}
