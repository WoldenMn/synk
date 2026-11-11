use crate::common::{memory_pool, test_vault};
use crate::support::fake_server::{fixture, FakeServer, Reply};
use sqlx::SqlitePool;
use std::sync::Arc;
use synk_lib::crypto::CryptoVault;
use synk_lib::sources::google_health::GoogleHealthSource;
use synk_lib::sources::oauth::TokenSet;
use synk_lib::sources::oura::OuraSource;
use synk_lib::sources::polar::PolarSource;
use synk_lib::sources::whoop::WhoopSource;
use synk_lib::sources::withings::WithingsSource;
use synk_lib::sources::{store, Domain, Source, SourceId, SourceKind, SyncContext, SyncFuture};

async fn connected(
    source: SourceId,
    access: &str,
    refresh: &str,
    expires_in: i64,
    user_ref: &str,
) -> (SqlitePool, Arc<CryptoVault>) {
    let pool = memory_pool().await;
    let vault = test_vault();
    store::set_credentials(&pool, vault.as_ref(), source, "client-test", "secret-test")
        .await
        .unwrap();
    let tokens = TokenSet {
        access_token: access.into(),
        refresh_token: refresh.into(),
        expires_in,
        token_type: "bearer".into(),
        user_ref: user_ref.into(),
    };
    store::save_auth(&pool, vault.as_ref(), source, &tokens)
        .await
        .unwrap();
    store::set_last_sync(&pool, source, "sleep", 1_700_000_000)
        .await
        .unwrap();
    (pool, vault)
}

async fn disconnected_locally(pool: &SqlitePool, source: SourceId) -> bool {
    !store::is_connected(pool, source).await.unwrap()
        && store::get_last_sync(pool, source, "sleep").await.unwrap() == 0
}

#[tokio::test]
async fn polar_is_deregistered_by_its_user_id_then_disconnected() {
    let (pool, vault) = connected(SourceId::POLAR, "access-test-1", "", 31_535_999, "10579").await;
    let fake = FakeServer::start(|_| (204, String::new()));
    let source = PolarSource::at(&fake.base_url);

    let report = synk_lib::services::sources::disconnect_with(&source, &pool, &vault)
        .await
        .unwrap();

    assert!(report.revoked);
    assert_eq!(report.warning, None);
    let calls = fake.calls_on("/v3/users/10579");
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].method, "DELETE");
    assert_eq!(
        calls[0].authorization.as_deref(),
        Some("Bearer access-test-1")
    );
    assert!(disconnected_locally(&pool, SourceId::POLAR).await);
}

#[tokio::test]
async fn whoop_access_is_revoked_with_the_current_token_then_disconnected() {
    let (pool, vault) =
        connected(SourceId::WHOOP, "access-test-2", "refresh-test-0", 3600, "").await;
    let fake = FakeServer::start(|_| (204, String::new()));
    let source = WhoopSource::at(&fake.base_url);

    let report = synk_lib::services::sources::disconnect_with(&source, &pool, &vault)
        .await
        .unwrap();

    assert!(report.revoked);
    let calls = fake.calls_on("/developer/v2/user/access");
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].method, "DELETE");
    assert_eq!(
        calls[0].authorization.as_deref(),
        Some("Bearer access-test-2")
    );
    assert!(disconnected_locally(&pool, SourceId::WHOOP).await);
}

#[tokio::test]
async fn whoop_refreshes_an_expired_access_token_before_revoking() {
    let (pool, vault) =
        connected(SourceId::WHOOP, "access-test-1", "refresh-test-0", -10, "").await;
    let fake = FakeServer::start_replying(|call| {
        if call.route() == "/oauth/oauth2/token" {
            return Reply::new(200, fixture("whoop", "token_refreshed.json"));
        }
        Reply::new(204, "")
    });
    let source = WhoopSource::at(&fake.base_url);

    let report = synk_lib::services::sources::disconnect_with(&source, &pool, &vault)
        .await
        .unwrap();

    assert!(report.revoked);
    let calls = fake.calls_on("/developer/v2/user/access");
    assert_eq!(
        calls[0].authorization.as_deref(),
        Some("Bearer access-test-2")
    );
}

#[tokio::test]
async fn google_revokes_the_refresh_token_which_ends_the_whole_grant() {
    let (pool, vault) = connected(
        SourceId::GOOGLE_HEALTH,
        "access-test-1",
        "refresh-test-0",
        3600,
        "",
    )
    .await;
    let fake = FakeServer::start(|_| (200, "{}".to_string()));
    let source = GoogleHealthSource::at(&fake.base_url);

    let report = synk_lib::services::sources::disconnect_with(&source, &pool, &vault)
        .await
        .unwrap();

    assert!(report.revoked);
    let calls = fake.calls_on("/revoke");
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].method, "POST");
    assert_eq!(
        calls[0].form_value("token").as_deref(),
        Some("refresh-test-0")
    );
    assert!(disconnected_locally(&pool, SourceId::GOOGLE_HEALTH).await);
}

#[tokio::test]
async fn a_refused_revocation_still_disconnects_locally_and_is_named_without_the_body() {
    let (pool, vault) = connected(SourceId::POLAR, "access-test-1", "", 31_535_999, "10579").await;
    let fake = FakeServer::start(|_| {
        (
            500,
            "refresh_token=RT-FUITE-1 access_token=AT-FUITE-1".to_string(),
        )
    });
    let source = PolarSource::at(&fake.base_url);

    let report = synk_lib::services::sources::disconnect_with(&source, &pool, &vault)
        .await
        .unwrap();

    assert!(!report.revoked);
    let warning = report.warning.expect("echec nomme");
    assert!(warning.contains("Polar"), "{warning}");
    assert!(warning.contains("500"), "{warning}");
    assert!(!warning.contains("FUITE"), "{warning}");
    assert!(disconnected_locally(&pool, SourceId::POLAR).await);
}

#[tokio::test]
async fn an_unreachable_brand_does_not_block_the_local_disconnection() {
    let (pool, vault) = connected(
        SourceId::GOOGLE_HEALTH,
        "access-test-1",
        "refresh-test-0",
        3600,
        "",
    )
    .await;
    let dead = {
        let fake = FakeServer::start(|_| (200, String::new()));
        fake.base_url.clone()
    };
    let source = GoogleHealthSource::at(dead);

    let report = synk_lib::services::sources::disconnect_with(&source, &pool, &vault)
        .await
        .unwrap();

    assert!(!report.revoked);
    assert!(report
        .warning
        .expect("echec nomme")
        .contains("Google Health"));
    assert!(disconnected_locally(&pool, SourceId::GOOGLE_HEALTH).await);
}

#[tokio::test]
async fn a_source_that_is_not_connected_calls_nobody() {
    let pool = memory_pool().await;
    let vault = test_vault();
    let fake = FakeServer::start(|_| (204, String::new()));
    let source = PolarSource::at(&fake.base_url);

    let report = synk_lib::services::sources::disconnect_with(&source, &pool, &vault)
        .await
        .unwrap();

    assert!(!report.revoked);
    assert_eq!(report.warning, None);
    assert!(fake.calls_to("/v3/users/10579").is_empty());
}

struct Mute;

impl Source for Mute {
    fn id(&self) -> SourceId {
        SourceId::OURA
    }

    fn label(&self) -> &'static str {
        "Muette"
    }

    fn kind(&self) -> SourceKind {
        SourceKind::OAuth(&synk_lib::sources::oura::OAUTH)
    }

    fn capabilities(&self) -> &'static [Domain] {
        &[]
    }

    fn sync<'a>(&'a self, _ctx: &'a SyncContext<'a>) -> SyncFuture<'a> {
        Box::pin(async { Ok(Default::default()) })
    }
}

#[tokio::test]
async fn a_brand_without_a_revocation_point_is_disconnected_with_no_warning() {
    let (pool, vault) =
        connected(SourceId::OURA, "access-test-1", "refresh-test-0", 3600, "").await;

    let report = synk_lib::services::sources::disconnect_with(&Mute, &pool, &vault)
        .await
        .unwrap();

    assert!(!report.revoked);
    assert_eq!(report.warning, None);
    assert!(disconnected_locally(&pool, SourceId::OURA).await);
}

#[tokio::test]
async fn oura_access_is_revoked_with_the_current_token_then_disconnected() {
    let (pool, vault) =
        connected(SourceId::OURA, "access-test-1", "refresh-test-0", 3600, "").await;
    let fake = FakeServer::start(|_| (200, "{}".to_string()));
    let source = OuraSource::at(&fake.base_url);

    let report = synk_lib::services::sources::disconnect_with(&source, &pool, &vault)
        .await
        .unwrap();

    assert!(report.revoked);
    assert_eq!(report.warning, None);
    let calls = fake.calls_on("/oauth/revoke");
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].method, "GET");
    assert_eq!(
        calls[0].query_value("access_token").as_deref(),
        Some("access-test-1")
    );
    assert!(disconnected_locally(&pool, SourceId::OURA).await);
}

#[tokio::test]
async fn a_refused_oura_revocation_is_named_without_the_body_and_still_disconnects() {
    let (pool, vault) =
        connected(SourceId::OURA, "access-test-1", "refresh-test-0", 3600, "").await;
    let fake = FakeServer::start(|_| (400, "access_token=AT-FUITE-1".to_string()));
    let source = OuraSource::at(&fake.base_url);

    let report = synk_lib::services::sources::disconnect_with(&source, &pool, &vault)
        .await
        .unwrap();

    assert!(!report.revoked);
    let warning = report.warning.expect("echec nomme");
    assert!(
        warning.contains("Oura") && warning.contains("400"),
        "{warning}"
    );
    assert!(
        !warning.contains("FUITE") && !warning.contains("access-test-1"),
        "{warning}"
    );
    assert!(disconnected_locally(&pool, SourceId::OURA).await);
}

#[tokio::test]
async fn withings_access_is_revoked_with_a_signed_nonce_then_disconnected() {
    let (pool, vault) = connected(
        SourceId::WITHINGS,
        "access-test-1",
        "refresh-test-0",
        3600,
        "4242",
    )
    .await;
    let fake = FakeServer::start_replying(|call| match call.route() {
        "/v2/signature" => Reply::new(200, r#"{"status":0,"body":{"nonce":"nonce-test"}}"#),
        _ => Reply::new(200, r#"{"status":0}"#),
    });
    let source = WithingsSource::at(&fake.base_url);

    let report = synk_lib::services::sources::disconnect_with(&source, &pool, &vault)
        .await
        .unwrap();

    assert!(report.revoked, "{:?}", report.warning);
    let nonce_call = &fake.calls_on("/v2/signature")[0];
    let timestamp = nonce_call.form_value("timestamp").expect("horodatage");
    assert_eq!(nonce_call.form_value("action").as_deref(), Some("getnonce"));
    assert_eq!(
        nonce_call.form_value("signature").unwrap(),
        synk_lib::utils::hmac::hmac_sha256_hex(
            b"secret-test",
            format!("getnonce,client-test,{timestamp}").as_bytes()
        )
    );
    let revoke_call = &fake.calls_on("/v2/oauth2")[0];
    assert_eq!(revoke_call.form_value("action").as_deref(), Some("revoke"));
    assert_eq!(revoke_call.form_value("userid").as_deref(), Some("4242"));
    assert_eq!(
        revoke_call.form_value("nonce").as_deref(),
        Some("nonce-test")
    );
    assert_eq!(
        revoke_call.form_value("signature").as_deref(),
        Some("d4ad6427581936dc68705d2055823a0fe40be2bcc10980a4c5215c69f73f9ea7")
    );
    assert!(disconnected_locally(&pool, SourceId::WITHINGS).await);
}

#[tokio::test]
async fn a_withings_refusal_is_a_warning_and_never_blocks_the_local_disconnection() {
    let (pool, vault) = connected(
        SourceId::WITHINGS,
        "access-test-1",
        "refresh-test-0",
        3600,
        "4242",
    )
    .await;
    let fake = FakeServer::start_replying(|call| match call.route() {
        "/v2/signature" => Reply::new(200, r#"{"status":0,"body":{"nonce":"nonce-test"}}"#),
        _ => Reply::new(200, r#"{"status":293,"error":"FUITE-signature"}"#),
    });
    let source = WithingsSource::at(&fake.base_url);

    let report = synk_lib::services::sources::disconnect_with(&source, &pool, &vault)
        .await
        .unwrap();

    assert!(!report.revoked);
    let warning = report.warning.expect("echec nomme");
    assert!(warning.contains("293"), "{warning}");
    assert!(!warning.contains("FUITE"), "{warning}");
    assert!(disconnected_locally(&pool, SourceId::WITHINGS).await);
}
