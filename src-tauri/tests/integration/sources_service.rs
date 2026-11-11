use crate::common::{memory_pool, test_vault};
use crate::support::import_gate;
use synk_lib::services::sources::{disconnect, list, set_credentials};
use synk_lib::sources::oauth::TokenSet;
use synk_lib::sources::{store, SourceId};

fn tokens() -> TokenSet {
    TokenSet {
        access_token: "AT".into(),
        refresh_token: "RT".into(),
        expires_in: 3600,
        token_type: "Bearer".into(),
        user_ref: "u1".into(),
    }
}

#[tokio::test]
async fn the_list_describes_every_registered_source() {
    let pool = memory_pool().await;
    let vault = test_vault();

    let sources = list(&pool, vault.as_ref()).await.expect("liste");

    assert_eq!(sources.len(), 9);
    let withings = &sources[0];
    assert_eq!(withings.id, "withings");
    assert_eq!(withings.label, "Withings");
    assert_eq!(withings.kind, "oauth");
    assert!(withings.domains.contains(&"body") && withings.domains.contains(&"sleep"));
    assert!(!withings.connected);
    assert!(!withings.credentials_configured);
    assert_eq!(withings.last_sync_at, None);
    assert_eq!(
        withings.redirect_uri.as_deref(),
        Some("http://localhost:17483/callback")
    );
    assert_eq!(
        withings.developer_url,
        Some("https://developer.withings.com/")
    );
    let oura = &sources[1];
    assert_eq!((oura.id, oura.label, oura.kind), ("oura", "Oura", "oauth"));
    assert_eq!(
        oura.domains,
        vec!["sleep", "activity", "workouts", "vitals"]
    );
    assert!(!oura.connected && !oura.credentials_configured);
    assert_eq!(
        oura.redirect_uri.as_deref(),
        Some("http://localhost:17483/callback/oura")
    );
    assert_eq!(
        oura.developer_url,
        Some("https://cloud.ouraring.com/oauth/applications")
    );
    let polar = &sources[2];
    assert_eq!(
        (polar.id, polar.label, polar.kind),
        ("polar", "Polar", "oauth")
    );
    assert_eq!(
        polar.domains,
        vec!["sleep", "activity", "workouts", "vitals"]
    );
    assert_eq!(
        polar.redirect_uri.as_deref(),
        Some("http://localhost:17483/callback/polar")
    );
    assert_eq!(
        polar.developer_url,
        Some("https://admin.polaraccesslink.com/")
    );
    let whoop = &sources[3];
    assert_eq!(
        (whoop.id, whoop.label, whoop.kind),
        ("whoop", "Whoop", "oauth")
    );
    assert_eq!(
        whoop.domains,
        vec!["sleep", "activity", "workouts", "vitals"]
    );
    assert_eq!(
        whoop.redirect_uri.as_deref(),
        Some("http://localhost:17483/callback/whoop")
    );
    assert_eq!(
        whoop.developer_url,
        Some("https://developer-dashboard.whoop.com/")
    );
    let google = &sources[4];
    assert_eq!(
        (google.id, google.label, google.kind),
        ("google_health", "Google Health", "oauth")
    );
    assert_eq!(
        google.redirect_uri.as_deref(),
        Some("http://localhost:17483/callback/google_health")
    );
    assert_eq!(
        google.developer_url,
        Some("https://console.developers.google.com/apis/library/health.googleapis.com")
    );
    assert!(google.reauth.is_none());
}

#[tokio::test]
async fn credentials_connection_and_last_sync_show_in_the_list() {
    let pool = memory_pool().await;
    let vault = test_vault();
    set_credentials(&pool, vault.as_ref(), "withings", "cid", "secret")
        .await
        .expect("identifiants");
    store::save_auth(&pool, vault.as_ref(), SourceId::WITHINGS, &tokens())
        .await
        .expect("jeton");
    sqlx::query(
        "INSERT INTO sync_log (source, started_at, finished_at, data_type, status) \
         VALUES ('withings', '2026-10-25 08:00:00', '2026-10-25 08:00:05', 'sleep', 'success'), \
                ('oura', '2026-10-26 08:00:00', '2026-10-26 08:00:05', 'sleep', 'success'), \
                ('withings', '2026-10-27 08:00:00', '2026-10-27 08:00:05', 'sleep', 'error')",
    )
    .execute(&pool)
    .await
    .expect("journal");

    let withings = list(&pool, vault.as_ref()).await.expect("liste").remove(0);

    assert!(withings.connected);
    assert!(withings.credentials_configured);
    assert_eq!(
        withings.last_sync_at.as_deref(),
        Some("2026-10-25T08:00:05Z")
    );
}

#[tokio::test]
async fn disconnecting_forgets_the_tokens_and_rewinds_the_cursors() {
    let pool = memory_pool().await;
    let vault = test_vault();
    store::save_auth(&pool, vault.as_ref(), SourceId::WITHINGS, &tokens())
        .await
        .expect("jeton");
    store::set_last_sync(&pool, SourceId::WITHINGS, "sleep", 1_700_000_000)
        .await
        .expect("curseur");

    let report = disconnect(&pool, &vault, "withings")
        .await
        .expect("deconnexion");
    assert!(!report.revoked);
    assert!(
        report.warning.is_some(),
        "sans identifiants la revocation est impossible et le dit"
    );

    assert!(!store::is_connected(&pool, SourceId::WITHINGS)
        .await
        .unwrap());
    assert_eq!(
        store::get_last_sync(&pool, SourceId::WITHINGS, "sleep")
            .await
            .unwrap(),
        0
    );
}

#[tokio::test]
async fn an_unknown_source_is_refused_not_ignored() {
    let pool = memory_pool().await;
    let vault = test_vault();

    let err = disconnect(&pool, &vault, "inconnue").await.unwrap_err();
    assert_eq!(err.kind(), "Validation");
    let err = set_credentials(&pool, vault.as_ref(), "inconnue", "a", "b")
        .await
        .unwrap_err();
    assert_eq!(err.kind(), "Validation");
}

#[tokio::test]
async fn credentials_are_trimmed_and_bounded_before_they_are_sealed() {
    let pool = memory_pool().await;
    let vault = test_vault();

    set_credentials(
        &pool,
        vault.as_ref(),
        "oura",
        "  id-propre  ",
        "\tsecret-propre\n",
    )
    .await
    .expect("espaces retires");
    let (id, secret) = store::get_credentials(&pool, vault.as_ref(), SourceId::OURA)
        .await
        .unwrap();
    assert_eq!(
        (id.as_str(), secret.as_str()),
        ("id-propre", "secret-propre")
    );

    for (client_id, client_secret) in [
        ("   ", "secret"),
        ("id", ""),
        (&"x".repeat(513)[..], "secret"),
        ("id", "se\ncret"),
    ] {
        let err = set_credentials(&pool, vault.as_ref(), "oura", client_id, client_secret)
            .await
            .unwrap_err();
        assert_eq!(err.kind(), "Validation", "{client_id:?}");
    }
    let (id, _) = store::get_credentials(&pool, vault.as_ref(), SourceId::OURA)
        .await
        .unwrap();
    assert_eq!(id.as_str(), "id-propre");
}

#[tokio::test]
async fn syncing_one_source_refuses_an_unknown_or_file_source_and_skips_a_disconnected_one() {
    use synk_lib::services::sources::sync_one;
    let _gate = import_gate::no_import_running().await;

    let pool = memory_pool().await;
    let vault = test_vault();

    let unknown = sync_one("inconnue", &pool, &vault, &|_| {})
        .await
        .unwrap_err();
    assert_eq!(unknown.kind(), "Validation");
    let file = sync_one("apple_health", &pool, &vault, &|_| {})
        .await
        .unwrap_err();
    assert_eq!(file.kind(), "Validation");
    assert!(file.to_string().contains("fichier"), "{file}");

    let outcomes = sync_one("oura", &pool, &vault, &|_| {}).await.unwrap();
    assert!(
        outcomes.is_empty(),
        "une source deconnectee ne synchronise rien"
    );
}
