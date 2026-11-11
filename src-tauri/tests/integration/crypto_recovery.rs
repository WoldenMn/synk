use std::sync::Arc;

use crate::common::get_pair;
use sqlx::SqlitePool;
use synk_lib::crypto::{
    invalidate_undecryptable_secrets, migrate_plaintext_secrets, CryptoVault, InvalidatedSecrets,
    InvalidatedSource,
};

const LOST_KEY: [u8; 32] = [0x11; 32];
const CURRENT_KEY: [u8; 32] = [0x22; 32];

fn hit(source: &str, label: &str, tokens: bool, credentials: bool) -> InvalidatedSource {
    InvalidatedSource {
        source: source.to_string(),
        label: label.to_string(),
        tokens,
        credentials,
    }
}

fn report_of(sources: Vec<InvalidatedSource>) -> InvalidatedSecrets {
    InvalidatedSecrets { sources }
}

async fn bootstrapped_pool(dir: &tempfile::TempDir) -> SqlitePool {
    synk_lib::db::init_pool(dir.path().join("synk.db"))
        .await
        .expect("base temporaire initialisee")
}

async fn insert_auth(pool: &SqlitePool, access: &str, refresh: &str) {
    sqlx::query(
        "INSERT INTO source_auth (source_id, access_token, refresh_token, expires_at, token_type) \
         VALUES ('withings', ?1, ?2, unixepoch('now') + 3600, 'Bearer')",
    )
    .bind(access)
    .bind(refresh)
    .execute(pool)
    .await
    .unwrap();
}

async fn set_credentials_raw(pool: &SqlitePool, cid: &str, secret: &str) {
    sqlx::query(
        "INSERT INTO source_credentials (source_id, client_id, client_secret) \
         VALUES ('withings', ?1, ?2) \
         ON CONFLICT(source_id) DO UPDATE SET \
         client_id = excluded.client_id, client_secret = excluded.client_secret",
    )
    .bind(cid)
    .bind(secret)
    .execute(pool)
    .await
    .unwrap();
}

async fn insert_health_data(pool: &SqlitePool) {
    sqlx::query(
        "INSERT INTO body_measures_legacy (measured_at, source, weight) \
         VALUES ('2026-09-01T08:00:00Z', 'manual', 72.5)",
    )
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO blood_pressure (measured_at, source, systolic, diastolic) \
         VALUES ('2026-09-01T08:00:00Z', 'manual', 125, 80)",
    )
    .execute(pool)
    .await
    .unwrap();
}

async fn count(pool: &SqlitePool, table: &str) -> i64 {
    sqlx::query_scalar(sqlx::AssertSqlSafe(format!("SELECT COUNT(*) FROM {table}")))
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn raw_auth(pool: &SqlitePool) -> Option<(String, String)> {
    sqlx::query_as("SELECT access_token, refresh_token FROM withings_auth WHERE id = 1")
        .fetch_optional(pool)
        .await
        .unwrap()
}

async fn raw_credentials(pool: &SqlitePool) -> (String, String) {
    sqlx::query_as("SELECT client_id, client_secret FROM withings_credentials WHERE id = 1")
        .fetch_one(pool)
        .await
        .unwrap()
}

#[tokio::test]
async fn secrets_sealed_with_a_lost_key_are_invalidated_and_health_data_kept() {
    let dir = tempfile::tempdir().unwrap();
    let pool = bootstrapped_pool(&dir).await;
    let lost = CryptoVault::from_key(LOST_KEY);
    let current = CryptoVault::from_key(CURRENT_KEY);

    insert_auth(
        &pool,
        &lost.seal_str("AT").unwrap(),
        &lost.seal_str("RT").unwrap(),
    )
    .await;
    set_credentials_raw(
        &pool,
        &lost.seal_str("cid").unwrap(),
        &lost.seal_str("secret").unwrap(),
    )
    .await;
    insert_health_data(&pool).await;
    let body_before = count(&pool, "body_measures_legacy").await;
    let bp_before = count(&pool, "blood_pressure").await;

    let report = invalidate_undecryptable_secrets(&pool, &current)
        .await
        .expect("un secret illisible ne doit pas faire echouer l'etape");

    assert_eq!(
        report,
        report_of(vec![hit("withings", "Withings", true, true)])
    );
    assert!(report.any());
    assert_eq!(
        raw_auth(&pool).await,
        None,
        "les tokens illisibles restent en base"
    );
    assert_eq!(
        raw_credentials(&pool).await,
        (String::new(), String::new()),
        "les credentials illisibles restent en base"
    );
    assert_eq!(count(&pool, "body_measures_legacy").await, body_before);
    assert_eq!(count(&pool, "blood_pressure").await, bp_before);
    assert!(
        body_before > 0 && bp_before > 0,
        "le montage doit contenir des mesures"
    );
}

#[tokio::test]
async fn after_invalidation_credentials_read_as_not_configured_and_sync_asks_to_reconnect() {
    let dir = tempfile::tempdir().unwrap();
    let pool = bootstrapped_pool(&dir).await;
    let lost = CryptoVault::from_key(LOST_KEY);
    let current = Arc::new(CryptoVault::from_key(CURRENT_KEY));

    insert_auth(
        &pool,
        &lost.seal_str("AT").unwrap(),
        &lost.seal_str("RT").unwrap(),
    )
    .await;
    set_credentials_raw(
        &pool,
        &lost.seal_str("cid").unwrap(),
        &lost.seal_str("secret").unwrap(),
    )
    .await;

    let before = get_pair(&pool, &current).await;
    assert!(
        before.as_ref().is_err_and(|e| e.kind() == "Crypto"),
        "le montage doit reproduire l'echec de dechiffrement, obtenu : {before:?}"
    );

    invalidate_undecryptable_secrets(&pool, &current)
        .await
        .unwrap();
    migrate_plaintext_secrets(&pool, &current).await.unwrap();

    let (id, secret) = get_pair(&pool, &current).await.unwrap();
    assert_eq!((id.as_str(), secret.as_str()), ("", ""));
    let sync = synk_lib::sources::withings::client::get_valid_client(&pool, &current).await;
    let err = sync
        .err()
        .expect("sans tokens, aucun client ne doit etre construit");
    assert_eq!(
        err.kind(),
        "Unauthorized",
        "attendu une demande de reconnexion, obtenu : {err}"
    );
}

#[tokio::test]
async fn readable_secrets_are_left_untouched() {
    let dir = tempfile::tempdir().unwrap();
    let pool = bootstrapped_pool(&dir).await;
    let current = CryptoVault::from_key(CURRENT_KEY);
    let at = current.seal_str("AT").unwrap();
    let rt = current.seal_str("RT").unwrap();
    let cid = current.seal_str("cid").unwrap();
    let secret = current.seal_str("secret").unwrap();
    insert_auth(&pool, &at, &rt).await;
    set_credentials_raw(&pool, &cid, &secret).await;

    let report = invalidate_undecryptable_secrets(&pool, &current)
        .await
        .unwrap();

    assert_eq!(report, InvalidatedSecrets::default());
    assert!(!report.any());
    assert_eq!(raw_auth(&pool).await, Some((at, rt)));
    assert_eq!(raw_credentials(&pool).await, (cid, secret));
}

#[tokio::test]
async fn plaintext_secrets_are_left_to_the_migration() {
    let dir = tempfile::tempdir().unwrap();
    let pool = bootstrapped_pool(&dir).await;
    let current = CryptoVault::from_key(CURRENT_KEY);
    insert_auth(&pool, "plain_at", "plain_rt").await;
    set_credentials_raw(&pool, "plain_cid", "plain_secret").await;

    let report = invalidate_undecryptable_secrets(&pool, &current)
        .await
        .unwrap();

    assert_eq!(report, InvalidatedSecrets::default());
    assert_eq!(
        raw_auth(&pool).await,
        Some(("plain_at".to_string(), "plain_rt".to_string()))
    );
    assert_eq!(
        raw_credentials(&pool).await,
        ("plain_cid".to_string(), "plain_secret".to_string())
    );
}

#[tokio::test]
async fn one_unreadable_token_invalidates_the_pair_and_keeps_readable_credentials() {
    let dir = tempfile::tempdir().unwrap();
    let pool = bootstrapped_pool(&dir).await;
    let lost = CryptoVault::from_key(LOST_KEY);
    let current = CryptoVault::from_key(CURRENT_KEY);
    insert_auth(
        &pool,
        &current.seal_str("AT").unwrap(),
        &lost.seal_str("RT").unwrap(),
    )
    .await;
    let cid = current.seal_str("cid").unwrap();
    let secret = current.seal_str("secret").unwrap();
    set_credentials_raw(&pool, &cid, &secret).await;

    let report = invalidate_undecryptable_secrets(&pool, &current)
        .await
        .unwrap();

    assert_eq!(
        report,
        report_of(vec![hit("withings", "Withings", true, false)])
    );
    assert_eq!(raw_auth(&pool).await, None);
    assert_eq!(raw_credentials(&pool).await, (cid, secret));
}

#[tokio::test]
async fn invalidation_is_idempotent_and_noop_without_secrets() {
    let dir = tempfile::tempdir().unwrap();
    let pool = bootstrapped_pool(&dir).await;
    let lost = CryptoVault::from_key(LOST_KEY);
    let current = CryptoVault::from_key(CURRENT_KEY);

    let empty = invalidate_undecryptable_secrets(&pool, &current)
        .await
        .unwrap();
    assert_eq!(empty, InvalidatedSecrets::default());

    set_credentials_raw(
        &pool,
        &lost.seal_str("cid").unwrap(),
        &lost.seal_str("secret").unwrap(),
    )
    .await;
    let first = invalidate_undecryptable_secrets(&pool, &current)
        .await
        .unwrap();
    let second = invalidate_undecryptable_secrets(&pool, &current)
        .await
        .unwrap();
    assert_eq!(
        first,
        report_of(vec![hit("withings", "Withings", false, true)])
    );
    assert_eq!(second, InvalidatedSecrets::default());
}

async fn insert_source_auth(pool: &SqlitePool, source: &str, access: &str, refresh: &str) {
    sqlx::query(
        "INSERT INTO source_auth (source_id, access_token, refresh_token, expires_at) \
         VALUES (?1, ?2, ?3, unixepoch('now') + 3600)",
    )
    .bind(source)
    .bind(access)
    .bind(refresh)
    .execute(pool)
    .await
    .unwrap();
}

async fn insert_source_credentials(pool: &SqlitePool, source: &str, cid: &str, secret: &str) {
    sqlx::query(
        "INSERT INTO source_credentials (source_id, client_id, client_secret) VALUES (?1, ?2, ?3)",
    )
    .bind(source)
    .bind(cid)
    .bind(secret)
    .execute(pool)
    .await
    .unwrap();
}

#[tokio::test]
async fn every_source_is_covered_and_named_not_only_withings() {
    let dir = tempfile::tempdir().unwrap();
    let pool = bootstrapped_pool(&dir).await;
    let lost = CryptoVault::from_key(LOST_KEY);
    let current = CryptoVault::from_key(CURRENT_KEY);

    insert_source_auth(
        &pool,
        "oura",
        &lost.seal_str("AT").unwrap(),
        &lost.seal_str("RT").unwrap(),
    )
    .await;
    insert_source_credentials(
        &pool,
        "oura",
        &lost.seal_str("cid").unwrap(),
        &lost.seal_str("secret").unwrap(),
    )
    .await;
    let polar_at = current.seal_str("AT").unwrap();
    let polar_rt = current.seal_str("RT").unwrap();
    insert_source_auth(&pool, "polar", &polar_at, &polar_rt).await;
    insert_auth(
        &pool,
        &lost.seal_str("AT").unwrap(),
        &lost.seal_str("RT").unwrap(),
    )
    .await;

    let report = invalidate_undecryptable_secrets(&pool, &current)
        .await
        .unwrap();

    let mut sources = report.sources.clone();
    sources.sort_by(|a, b| a.source.cmp(&b.source));
    assert_eq!(
        sources,
        vec![
            hit("oura", "Oura", true, true),
            hit("withings", "Withings", true, false),
        ]
    );
    let left: Vec<String> = sqlx::query_scalar("SELECT source_id FROM source_auth ORDER BY 1")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(left, vec!["polar".to_string()], "la source lisible reste");
    let polar: (String, String) = sqlx::query_as(
        "SELECT access_token, refresh_token FROM source_auth WHERE source_id = 'polar'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(polar, (polar_at, polar_rt));
    let oura_creds: (String, String) = sqlx::query_as(
        "SELECT client_id, client_secret FROM source_credentials WHERE source_id = 'oura'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(oura_creds, (String::new(), String::new()));
    assert_eq!(raw_auth(&pool).await, None);
}
