use crate::common::{memory_pool, test_vault};
use synk_lib::sources::oauth::TokenSet;
use synk_lib::sources::{store, SourceId};

const OURA: SourceId = SourceId::new("oura");

fn tokens(access: &str, refresh: &str) -> TokenSet {
    TokenSet {
        access_token: access.into(),
        refresh_token: refresh.into(),
        expires_in: 3600,
        token_type: "Bearer".into(),
        user_ref: "u1".into(),
    }
}

#[tokio::test]
async fn a_row_of_the_source_tables_is_read_through_the_store_and_the_withings_views() {
    let pool = memory_pool().await;
    let vault = test_vault();
    sqlx::query("DELETE FROM source_credentials")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO source_auth (source_id, access_token, refresh_token, expires_at, user_ref) \
         VALUES ('withings', ?1, ?2, ?3, '4242')",
    )
    .bind(vault.seal_str("AT-OLD").unwrap())
    .bind(vault.seal_str("RT-OLD").unwrap())
    .bind(chrono::Utc::now().timestamp() + 10_800)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO source_credentials (source_id, client_id, client_secret) VALUES ('withings', ?1, ?2)",
    )
    .bind(vault.seal_str("old-cid").unwrap())
    .bind(vault.seal_str("old-secret").unwrap())
    .execute(&pool)
    .await
    .unwrap();

    let stored = store::load_auth(&pool, vault.as_ref(), SourceId::WITHINGS)
        .await
        .unwrap()
        .expect("jeton relu");
    assert_eq!(stored.access_token.as_str(), "AT-OLD");
    assert_eq!(stored.refresh_token.as_str(), "RT-OLD");
    assert!(synk_lib::sources::withings::auth::is_connected_inner(&pool)
        .await
        .unwrap());
    let (cid, secret) = store::get_credentials(&pool, vault.as_ref(), SourceId::WITHINGS)
        .await
        .unwrap();
    assert_eq!((cid.as_str(), secret.as_str()), ("old-cid", "old-secret"));

    let (id, user): (i64, String) =
        sqlx::query_as("SELECT id, withings_user_id FROM withings_auth")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!((id, user.as_str()), (1, "4242"));
    let view_cid: String =
        sqlx::query_scalar("SELECT client_id FROM withings_credentials WHERE id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(vault.open_str(&view_cid).unwrap().as_str(), "old-cid");
}

#[tokio::test]
async fn saving_a_token_writes_the_source_table_the_views_read() {
    let pool = memory_pool().await;
    let vault = test_vault();

    store::save_auth(
        &pool,
        vault.as_ref(),
        SourceId::WITHINGS,
        &tokens("AT-NEW", "RT-NEW"),
    )
    .await
    .unwrap();

    let stored = store::load_auth(&pool, vault.as_ref(), SourceId::WITHINGS)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored.access_token.as_str(), "AT-NEW");
    let in_view: String = sqlx::query_scalar("SELECT access_token FROM withings_auth WHERE id = 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(vault.open_str(&in_view).unwrap().as_str(), "AT-NEW");
}

#[tokio::test]
async fn disconnecting_clears_the_tokens_so_nothing_comes_back() {
    let pool = memory_pool().await;
    let vault = test_vault();
    store::save_auth(
        &pool,
        vault.as_ref(),
        SourceId::WITHINGS,
        &tokens("AT-NEW", "RT-NEW"),
    )
    .await
    .unwrap();

    synk_lib::sources::withings::auth::disconnect_inner(&pool)
        .await
        .unwrap();

    assert!(store::load_auth(&pool, vault.as_ref(), SourceId::WITHINGS)
        .await
        .unwrap()
        .is_none());
    assert!(!store::is_connected(&pool, SourceId::WITHINGS)
        .await
        .unwrap());
    let left: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM withings_auth")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(left, 0);
}

#[tokio::test]
async fn another_source_round_trips_through_the_source_tables_encrypted() {
    let pool = memory_pool().await;
    let vault = test_vault();

    store::save_auth(&pool, vault.as_ref(), OURA, &tokens("AT-O", "RT-O"))
        .await
        .unwrap();
    store::set_credentials(&pool, vault.as_ref(), OURA, "oura-cid", "oura-secret")
        .await
        .unwrap();

    let raw: (String, String) = sqlx::query_as(
        "SELECT access_token, refresh_token FROM source_auth WHERE source_id = 'oura'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(
        raw.0.starts_with("enc:v1:") && raw.1.starts_with("enc:v1:"),
        "{raw:?}"
    );
    let raw_creds: (String, String) = sqlx::query_as(
        "SELECT client_id, client_secret FROM source_credentials WHERE source_id = 'oura'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(raw_creds.0.starts_with("enc:v1:"), "{raw_creds:?}");

    let stored = store::load_auth(&pool, vault.as_ref(), OURA)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored.refresh_token.as_str(), "RT-O");
    let (cid, secret) = store::get_credentials(&pool, vault.as_ref(), OURA)
        .await
        .unwrap();
    assert_eq!((cid.as_str(), secret.as_str()), ("oura-cid", "oura-secret"));
    assert!(store::load_auth(&pool, vault.as_ref(), SourceId::WITHINGS)
        .await
        .unwrap()
        .is_none());
}

#[tokio::test]
async fn a_source_absent_from_the_sources_table_cannot_store_secrets() {
    let pool = crate::common::memory_pool_with_foreign_keys().await;
    let vault = test_vault();
    let err = store::save_auth(
        &pool,
        vault.as_ref(),
        SourceId::new("inconnue"),
        &tokens("A", "R"),
    )
    .await
    .unwrap_err();
    assert!(err.to_string().to_lowercase().contains("foreign"), "{err}");
}

#[tokio::test]
async fn missing_credentials_name_the_source_and_the_field() {
    let pool = memory_pool().await;
    let vault = test_vault();
    let err = store::require_client_id(&pool, vault.as_ref(), OURA)
        .await
        .unwrap_err();
    assert_eq!(err.kind(), "Validation");
    assert!(err.to_string().contains("Client ID"), "{err}");
    assert!(err.to_string().to_lowercase().contains("oura"), "{err}");
}

#[tokio::test]
#[ignore = "lit une copie MIGREE de la base de dev : SYNK_R33_DEV_COPY=<chemin> et la cle du service synk-dev"]
async fn credentials_of_a_migrated_dev_copy_are_read_through_the_store() {
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

    let path = std::env::var("SYNK_R33_DEV_COPY").expect("SYNK_R33_DEV_COPY");
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(SqliteConnectOptions::new().filename(&path).read_only(true))
        .await
        .unwrap();
    let vault = synk_lib::crypto::CryptoVault::init("synk-dev").unwrap();

    let raw: String =
        sqlx::query_scalar("SELECT client_id FROM source_credentials WHERE source_id = 'withings'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(raw.starts_with("enc:v1:"), "{raw}");
    let through_view: String =
        sqlx::query_scalar("SELECT client_id FROM withings_credentials WHERE id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(raw, through_view);
    store::get_credentials(&pool, &vault, SourceId::WITHINGS)
        .await
        .expect("la ligne chiffree par l'ancien code se dechiffre par le magasin");
}

#[tokio::test]
async fn the_connection_date_survives_a_token_refresh_and_restarts_on_a_new_connection() {
    let pool = memory_pool().await;
    let vault = test_vault();
    store::save_auth(&pool, vault.as_ref(), OURA, &tokens("AT1", "RT1"))
        .await
        .unwrap();
    sqlx::query(
        "UPDATE source_auth SET created_at = '2026-01-01 00:00:00' WHERE source_id = 'oura'",
    )
    .execute(&pool)
    .await
    .unwrap();

    store::save_auth(&pool, vault.as_ref(), OURA, &tokens("AT2", "RT2"))
        .await
        .unwrap();
    let kept: String =
        sqlx::query_scalar("SELECT created_at FROM source_auth WHERE source_id = 'oura'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(kept, "2026-01-01 00:00:00");

    store::delete_auth(&pool, OURA).await.unwrap();
    store::save_auth(&pool, vault.as_ref(), OURA, &tokens("AT3", "RT3"))
        .await
        .unwrap();
    let renewed: String =
        sqlx::query_scalar("SELECT created_at FROM source_auth WHERE source_id = 'oura'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_ne!(renewed, "2026-01-01 00:00:00");
}
