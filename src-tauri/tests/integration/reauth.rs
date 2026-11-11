use crate::common::{memory_pool, memory_pool_with_source, test_vault};
use crate::support::fake_server::{Call, FakeServer};
use sqlx::SqlitePool;
use std::sync::Arc;
use synk_lib::crypto::CryptoVault;
use synk_lib::services::sources::{disconnect, list_at};
use synk_lib::sources::oauth::reauth::REFRESH_EXPIRY;
use synk_lib::sources::oauth::token::standard_json;
use synk_lib::sources::oauth::{exchange_and_store, refresh_locked, ClientAuth, OAuthSpec};
use synk_lib::sources::{store, SourceId};

const WEEK: i64 = 7 * 86_400;
const FRESH: &str =
    r#"{"access_token":"AT-1","refresh_token":"RT-1","expires_in":3600,"token_type":"Bearer"}"#;

static SHORT_LIVED: OAuthSpec = OAuthSpec {
    id: SourceId::new("acme"),
    authorize_url: "https://acme.test/authorize",
    api_base: "",
    token_path: "/token",
    scopes: "read",
    pkce: false,
    redirect_path: None,
    token_params: &[],
    authorize_params: &[],
    refresh_lifetime_secs: Some(WEEK),
    parse_token: standard_json,
    client_auth: ClientAuth::Body,
};

static LONG_LIVED: OAuthSpec = OAuthSpec {
    refresh_lifetime_secs: None,
    ..SHORT_LIVED
};

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

async fn with_credentials(spec: &OAuthSpec) -> (SqlitePool, Arc<CryptoVault>) {
    let pool = memory_pool_with_source("acme").await;
    let vault = test_vault();
    store::set_credentials(&pool, vault.as_ref(), spec.id, "cid", "csecret")
        .await
        .unwrap();
    (pool, vault)
}

async fn expiry(pool: &SqlitePool, id: SourceId) -> i64 {
    store::get_last_sync(pool, id, REFRESH_EXPIRY)
        .await
        .unwrap()
}

#[tokio::test]
async fn a_connection_notes_the_refresh_token_expiry_when_the_spec_declares_one() {
    let (pool, vault) = with_credentials(&SHORT_LIVED).await;
    let fake = FakeServer::start(|_: &Call| (200, FRESH.to_string()));
    let before = now();

    exchange_and_store(
        &SHORT_LIVED,
        &fake.base_url,
        &pool,
        vault.as_ref(),
        "code",
        None,
    )
    .await
    .unwrap();

    let noted = expiry(&pool, SHORT_LIVED.id).await;
    assert!(noted >= before + WEEK && noted <= now() + WEEK, "{noted}");
}

#[tokio::test]
async fn a_spec_without_a_lifetime_notes_nothing() {
    let (pool, vault) = with_credentials(&LONG_LIVED).await;
    let fake = FakeServer::start(|_: &Call| (200, FRESH.to_string()));

    exchange_and_store(
        &LONG_LIVED,
        &fake.base_url,
        &pool,
        vault.as_ref(),
        "code",
        None,
    )
    .await
    .unwrap();

    assert_eq!(expiry(&pool, LONG_LIVED.id).await, 0);
}

async fn refreshed_with_noted_expiry(noted: i64) -> i64 {
    let (pool, vault) = with_credentials(&SHORT_LIVED).await;
    let fake = FakeServer::start(|_: &Call| (200, FRESH.to_string()));
    let tokens = synk_lib::sources::oauth::TokenSet {
        access_token: "AT-0".into(),
        refresh_token: "RT-0".into(),
        expires_in: -100,
        token_type: "Bearer".into(),
        user_ref: String::new(),
    };
    store::save_auth(&pool, vault.as_ref(), SHORT_LIVED.id, &tokens)
        .await
        .unwrap();
    store::set_last_sync(&pool, SHORT_LIVED.id, REFRESH_EXPIRY, noted)
        .await
        .unwrap();

    refresh_locked(&SHORT_LIVED, &fake.base_url, &pool, vault.as_ref(), "AT-0")
        .await
        .unwrap();

    expiry(&pool, SHORT_LIVED.id).await
}

#[tokio::test]
async fn a_refresh_that_works_after_the_noted_expiry_proves_the_assumption_wrong_and_clears_it() {
    assert_eq!(refreshed_with_noted_expiry(now() - 3600).await, 0);
}

#[tokio::test]
async fn a_refresh_before_the_noted_expiry_keeps_it() {
    let noted = now() + 3600;
    assert_eq!(refreshed_with_noted_expiry(noted).await, noted);
}

#[tokio::test]
async fn the_source_list_announces_the_reconnection_the_day_before_on_an_injected_clock() {
    let pool = memory_pool().await;
    let vault = test_vault();
    let expires = 1_800_000_000;
    store::set_last_sync(&pool, SourceId::OURA, REFRESH_EXPIRY, expires)
        .await
        .unwrap();
    let find = |all: Vec<synk_lib::services::sources::SourceInfo>| {
        all.into_iter().find(|s| s.id == "oura").unwrap()
    };

    let early = find(
        list_at(&pool, vault.as_ref(), expires - 2 * 86_400)
            .await
            .unwrap(),
    );
    assert!(early.reauth.is_none());

    let eve = find(
        list_at(&pool, vault.as_ref(), expires - 86_000)
            .await
            .unwrap(),
    );
    let notice = eve.reauth.expect("annonce a J-1");
    assert_eq!((notice.status, notice.expires_at), ("soon", expires));

    let late = find(list_at(&pool, vault.as_ref(), expires + 10).await.unwrap());
    assert_eq!(late.reauth.expect("expire").status, "expired");

    disconnect(&pool, &vault, "oura").await.unwrap();
    let after = find(list_at(&pool, vault.as_ref(), expires + 10).await.unwrap());
    assert!(after.reauth.is_none());
}
