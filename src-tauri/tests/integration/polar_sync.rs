use crate::common::{memory_pool, test_vault};
use crate::support::fake_server::{fixture, Call, FakeServer, Reply};
use base64::Engine;
use sqlx::SqlitePool;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use synk_lib::crypto::CryptoVault;
use synk_lib::error::AppError;
use synk_lib::sources::oauth::{exchange_and_store, TokenSet};
use synk_lib::sources::polar::{PolarSource, OAUTH};
use synk_lib::sources::sync::{sync_sources, Progress};
use synk_lib::sources::{store, Domain, Source, SourceId, SyncBatch, SyncContext};

const GOOD_TOKEN: &str = "Bearer access-test-1";

fn api(call: &Call, accepted: &str) -> Reply {
    if call.authorization.as_deref() != Some(accepted) {
        return Reply::new(401, fixture("polar", "error_401.json"));
    }
    let name = match call.route() {
        "/v3/users" => return Reply::new(200, fixture("polar", "user.json")),
        "/v3/users/sleep" => "nights.json",
        "/v3/users/nightly-recharge" => "recharges.json",
        "/v3/users/activities" => "activities.json",
        "/v3/exercises" => "exercises.json",
        _ => return Reply::new(404, "{}"),
    };
    Reply::new(200, fixture("polar", name))
}

fn healthy() -> FakeServer {
    FakeServer::start_replying(|call| api(call, GOOD_TOKEN))
}

async fn connected(expires_in: i64) -> (SqlitePool, Arc<CryptoVault>) {
    let pool = memory_pool().await;
    let vault = test_vault();
    store::set_credentials(
        &pool,
        vault.as_ref(),
        SourceId::POLAR,
        "client-test",
        "secret-test",
    )
    .await
    .unwrap();
    let tokens = TokenSet {
        access_token: "access-test-1".into(),
        refresh_token: String::new(),
        expires_in,
        token_type: "bearer".into(),
        user_ref: "10579".into(),
    };
    store::save_auth(&pool, vault.as_ref(), SourceId::POLAR, &tokens)
        .await
        .unwrap();
    (pool, vault)
}

async fn run(
    pool: &SqlitePool,
    vault: &Arc<CryptoVault>,
    base: &str,
) -> Result<SyncBatch, AppError> {
    let source = PolarSource::at(base);
    let ctx = SyncContext {
        pool,
        vault,
        on_progress: &|_, _| {},
    };
    source.sync(&ctx).await
}

async fn count(pool: &SqlitePool, sql: &'static str) -> i64 {
    sqlx::query_scalar(sql).fetch_one(pool).await.unwrap()
}

async fn totals(pool: &SqlitePool) -> (i64, i64, i64, i64) {
    (
        count(
            pool,
            "SELECT COUNT(*) FROM sleep_records WHERE source = 'polar'",
        )
        .await,
        count(pool, "SELECT COUNT(*) FROM activity WHERE source = 'polar'").await,
        count(pool, "SELECT COUNT(*) FROM workouts WHERE source = 'polar'").await,
        count(
            pool,
            "SELECT COUNT(*) FROM vitals_legacy WHERE source = 'polar'",
        )
        .await,
    )
}

fn day(offset: i64) -> String {
    (chrono::Utc::now() + chrono::Duration::days(offset))
        .format("%Y-%m-%d")
        .to_string()
}

#[tokio::test]
async fn the_code_exchange_authenticates_the_client_by_basic_header_then_the_source_syncs() {
    let pool = memory_pool().await;
    let vault = test_vault();
    store::set_credentials(
        &pool,
        vault.as_ref(),
        SourceId::POLAR,
        "client-test",
        "secret-test",
    )
    .await
    .unwrap();
    assert!(!store::is_connected(&pool, SourceId::POLAR).await.unwrap());
    let fake = FakeServer::start_replying(|call| {
        if call.route() == "/v2/oauth2/token" {
            return Reply::new(200, fixture("polar", "token.json"));
        }
        api(call, GOOD_TOKEN)
    });

    exchange_and_store(
        &OAUTH,
        &fake.base_url,
        &pool,
        vault.as_ref(),
        "code-test",
        None,
    )
    .await
    .unwrap();

    let call = &fake.calls_on("/v2/oauth2/token")[0];
    let basic = format!(
        "Basic {}",
        base64::engine::general_purpose::STANDARD.encode("client-test:secret-test")
    );
    assert_eq!(call.authorization.as_deref(), Some(basic.as_str()));
    assert_eq!(call.form_value("client_secret"), None);
    assert_eq!(call.form_value("code").as_deref(), Some("code-test"));
    assert_eq!(
        call.form_value("redirect_uri").as_deref(),
        Some("http://localhost:17483/callback/polar")
    );
    assert_eq!(call.form_value("code_verifier"), None);
    assert!(store::is_connected(&pool, SourceId::POLAR).await.unwrap());
    let stored = store::load_auth(&pool, vault.as_ref(), SourceId::POLAR)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored.access_token.as_str(), "access-test-1");
    assert_eq!(stored.user_ref, "10579");

    let source = PolarSource::at(&fake.base_url);
    let events = std::sync::Mutex::new(Vec::new());
    let on_progress = |p: Progress| {
        events
            .lock()
            .unwrap()
            .push(format!("{}:{}/{}", p.source, p.etape, p.total))
    };
    let outcomes = sync_sources(&[&source], &pool, &vault, &on_progress).await;
    assert_eq!(outcomes.len(), 1);
    assert!(!outcomes[0].failed, "{:?}", outcomes[0].errors);
    assert_eq!(outcomes[0].label, "Polar");
    assert_eq!(
        events.into_inner().unwrap(),
        vec!["polar:1/3", "polar:2/3", "polar:3/3", "analyses:1/1"]
    );
}

#[tokio::test]
async fn a_first_sync_registers_the_user_once_then_writes_every_domain() {
    let (pool, vault) = connected(31_535_999).await;
    let fake = healthy();

    let batch = run(&pool, &vault, &fake.base_url).await.unwrap();

    assert!(batch.errors.is_empty(), "{:?}", batch.errors);
    let registrations = fake.calls_on("/v3/users");
    assert_eq!(registrations.len(), 1);
    assert_eq!(registrations[0].authorization.as_deref(), Some(GOOD_TOKEN));
    assert!(
        registrations[0]
            .form
            .contains("\"member-id\":\"synk-10579\""),
        "{}",
        registrations[0].form
    );
    assert!(
        store::get_last_sync(&pool, SourceId::POLAR, "registration")
            .await
            .unwrap()
            > 0
    );

    type SleepLine = (String, Option<i64>, Option<i64>, Option<f64>, Option<i64>);
    let sleeps: Vec<SleepLine> = sqlx::query_as(
        "SELECT date, duration_total, sleep_score, rr_average, hr_min FROM sleep_records \
         WHERE source = 'polar' ORDER BY date",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        sleeps,
        vec![
            (
                "2026-10-24".into(),
                Some(25500),
                Some(83),
                Some(14.1),
                Some(50)
            ),
            (
                "2026-10-25".into(),
                Some(21000),
                Some(71),
                Some(13.8),
                Some(54)
            ),
        ]
    );
    let activity: (i64, Option<i64>, Option<i64>) = sqlx::query_as(
        "SELECT COUNT(*), MAX(steps), MAX(duration_active) FROM activity WHERE source = 'polar'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(activity, (2, Some(8823), Some(11460)));
    let workout: (String, i64, String, Option<i64>) = sqlx::query_as(
        "SELECT category_label, duration, start_at, hr_max FROM workouts WHERE source = 'polar'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        workout,
        (
            "running".into(),
            2730,
            "2026-10-24T16:00:00Z".into(),
            Some(171)
        )
    );
    for (kind, expected) in [
        ("hrv", 2),
        ("ans_charge", 2),
        ("nightly_recharge_status", 2),
    ] {
        let n: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM vitals_legacy WHERE source = 'polar' AND vital_type = ?1",
        )
        .bind(kind)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(n, expected, "{kind}");
    }
    assert_eq!(batch.written_in(Domain::Sleep), 2);
    assert_eq!(batch.written_in(Domain::Activity), 2);
    assert_eq!(batch.written_in(Domain::Workouts), 1);
    assert_eq!(batch.written_in(Domain::Vitals), 6);

    let activity_calls = fake.calls_on("/v3/users/activities");
    assert_eq!(activity_calls.len(), 7);
    assert_eq!(activity_calls[0].query_value("from"), Some(day(-180)));
    assert_eq!(activity_calls[6].query_value("to"), Some(day(0)));
    assert!(fake.calls_on("/v2/oauth2/token").is_empty());
}

#[tokio::test]
async fn a_user_already_registered_is_not_an_error_and_is_not_registered_again() {
    let (pool, vault) = connected(31_535_999).await;
    let fake = FakeServer::start_replying(|call| {
        if call.route() == "/v3/users" {
            return Reply::new(409, fixture("polar", "error_409.json"));
        }
        api(call, GOOD_TOKEN)
    });

    let first = run(&pool, &vault, &fake.base_url).await.unwrap();
    let second = run(&pool, &vault, &fake.base_url).await.unwrap();

    assert!(first.errors.is_empty(), "{:?}", first.errors);
    assert!(second.errors.is_empty(), "{:?}", second.errors);
    assert_eq!(fake.calls_on("/v3/users").len(), 1);
    assert_eq!(totals(&pool).await, (2, 2, 1, 6));
}

#[tokio::test]
async fn an_incremental_sync_restarts_a_week_back_and_duplicates_nothing() {
    let (pool, vault) = connected(31_535_999).await;
    let fake = healthy();
    run(&pool, &vault, &fake.base_url).await.unwrap();
    let before = totals(&pool).await;

    let second = run(&pool, &vault, &fake.base_url).await.unwrap();

    assert_eq!(before, totals(&pool).await);
    assert_eq!(second.written_in(Domain::Vitals), 0);
    let activity_calls = fake.calls_on("/v3/users/activities");
    assert_eq!(activity_calls.len(), 8);
    assert_eq!(activity_calls[7].query_value("from"), Some(day(-7)));
    assert_eq!(fake.calls_on("/v3/users").len(), 1);
    assert!(
        store::get_last_sync(&pool, SourceId::POLAR, "activity")
            .await
            .unwrap()
            > 0
    );
}

#[tokio::test]
async fn a_token_past_its_announced_expiry_is_used_as_is_polar_has_no_refresh() {
    let (pool, vault) = connected(-100).await;
    let fake = healthy();

    let batch = run(&pool, &vault, &fake.base_url).await.unwrap();

    assert!(batch.errors.is_empty(), "{:?}", batch.errors);
    assert!(fake.calls_on("/v2/oauth2/token").is_empty());
    assert_eq!(totals(&pool).await, (2, 2, 1, 6));
}

#[tokio::test]
async fn a_revoked_token_stops_the_source_as_unauthorized_and_writes_nothing() {
    let (pool, vault) = connected(31_535_999).await;
    let fake = FakeServer::start_replying(|call| api(call, "Bearer autre-jeton"));

    let err = run(&pool, &vault, &fake.base_url).await.unwrap_err();

    assert_eq!(err.kind(), "Unauthorized");
    assert!(!err.to_string().contains("FUITE"), "{err}");
    assert_eq!(totals(&pool).await, (0, 0, 0, 0));
    assert_eq!(
        store::get_last_sync(&pool, SourceId::POLAR, "registration")
            .await
            .unwrap(),
        0
    );
    assert!(fake.calls_on("/v2/oauth2/token").is_empty());
}

#[tokio::test]
async fn missing_consents_at_registration_are_named_and_nothing_is_read() {
    let (pool, vault) = connected(31_535_999).await;
    let fake = FakeServer::start_replying(|call| {
        if call.route() == "/v3/users" {
            return Reply::new(403, fixture("polar", "error_403.json"));
        }
        api(call, GOOD_TOKEN)
    });

    let err = run(&pool, &vault, &fake.base_url).await.unwrap_err();

    assert_eq!(err.kind(), "Forbidden");
    assert!(err.to_string().contains("consentements"), "{err}");
    assert!(!err.to_string().contains("FUITE"), "{err}");
    assert!(fake.calls_on("/v3/users/sleep").is_empty());
}

#[tokio::test]
async fn a_rate_limited_call_waits_for_the_reset_header_and_then_succeeds() {
    let (pool, vault) = connected(31_535_999).await;
    let hits = Arc::new(AtomicUsize::new(0));
    let seen = hits.clone();
    let fake = FakeServer::start_replying(move |call| {
        if call.route() == "/v3/exercises" && seen.fetch_add(1, Ordering::SeqCst) == 0 {
            return Reply::new(429, fixture("polar", "error_429.json"))
                .with_header("RateLimit-Reset", "0, 3600");
        }
        api(call, GOOD_TOKEN)
    });

    let batch = run(&pool, &vault, &fake.base_url).await.unwrap();

    assert!(batch.errors.is_empty(), "{:?}", batch.errors);
    assert_eq!(fake.calls_on("/v3/exercises").len(), 2);
    assert_eq!(totals(&pool).await.2, 1);
}

#[tokio::test]
async fn a_rate_limit_that_never_lifts_costs_one_step_and_spares_the_others() {
    let (pool, vault) = connected(31_535_999).await;
    let fake = FakeServer::start_replying(|call| {
        if call.route() == "/v3/users/activities" {
            return Reply::new(429, fixture("polar", "error_429.json"))
                .with_header("Retry-After", "0");
        }
        api(call, GOOD_TOKEN)
    });

    let batch = run(&pool, &vault, &fake.base_url).await.unwrap();

    assert_eq!(fake.calls_on("/v3/users/activities").len(), 3);
    assert_eq!(batch.errors.len(), 1, "{:?}", batch.errors);
    assert!(
        batch.errors[0].starts_with("Activité : "),
        "{:?}",
        batch.errors
    );
    assert!(batch.errors[0].contains("Limite"), "{:?}", batch.errors);
    assert_eq!(totals(&pool).await, (2, 0, 1, 6));
    assert_eq!(
        store::get_last_sync(&pool, SourceId::POLAR, "activity")
            .await
            .unwrap(),
        0
    );
}

#[tokio::test]
async fn a_failed_write_keeps_the_cursor_and_the_next_sync_fetches_the_data_again() {
    let (pool, vault) = connected(31_535_999).await;
    let fake = healthy();
    sqlx::query(
        "CREATE TRIGGER refuse_workouts BEFORE INSERT ON workouts \
         BEGIN SELECT RAISE(ABORT, 'ecriture refusee'); END",
    )
    .execute(&pool)
    .await
    .unwrap();

    let failed = run(&pool, &vault, &fake.base_url).await.unwrap();

    assert_eq!(failed.errors.len(), 1, "{:?}", failed.errors);
    assert!(
        failed.errors[0].starts_with("Entraînements : "),
        "{:?}",
        failed.errors
    );
    assert_eq!(totals(&pool).await, (2, 2, 0, 6));
    assert_eq!(
        store::get_last_sync(&pool, SourceId::POLAR, "workouts")
            .await
            .unwrap(),
        0
    );
    assert!(
        store::get_last_sync(&pool, SourceId::POLAR, "sleep")
            .await
            .unwrap()
            > 0
    );

    sqlx::query("DROP TRIGGER refuse_workouts")
        .execute(&pool)
        .await
        .unwrap();
    let healed = run(&pool, &vault, &fake.base_url).await.unwrap();

    assert!(healed.errors.is_empty(), "{:?}", healed.errors);
    assert_eq!(fake.calls_on("/v3/exercises").len(), 2);
    assert_eq!(totals(&pool).await, (2, 2, 1, 6));
    assert!(
        store::get_last_sync(&pool, SourceId::POLAR, "workouts")
            .await
            .unwrap()
            > 0
    );
}

#[tokio::test]
async fn a_server_error_never_copies_the_body_into_the_message() {
    let (pool, vault) = connected(31_535_999).await;
    let fake = FakeServer::start_replying(|call| {
        if call.route() == "/v3/exercises" {
            return Reply::new(500, "access_token=AT-FUITE-9 refresh_token=RT-FUITE-9");
        }
        api(call, GOOD_TOKEN)
    });

    let batch = run(&pool, &vault, &fake.base_url).await.unwrap();

    assert_eq!(batch.errors.len(), 1, "{:?}", batch.errors);
    assert!(
        batch.errors[0].starts_with("Entraînements : "),
        "{:?}",
        batch.errors
    );
    assert!(!batch.errors[0].contains("FUITE"), "{:?}", batch.errors);
    assert_eq!(fake.calls_on("/v3/exercises").len(), 3);
}

#[tokio::test]
async fn the_bootstrap_declares_the_polar_metrics_once_even_when_replayed() {
    let pool = memory_pool().await;
    synk_lib::db::bootstrap_schema(&pool).await.unwrap();
    synk_lib::db::bootstrap_schema(&pool).await.unwrap();
    let rows: Vec<(String, String, f64, f64)> = sqlx::query_as(
        "SELECT slug, category, min_value, max_value FROM metric_definitions \
         WHERE slug IN ('ans_charge', 'nightly_recharge_status') ORDER BY slug",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        rows,
        vec![
            ("ans_charge".into(), "body_scan".into(), -10.0, 10.0),
            (
                "nightly_recharge_status".into(),
                "body_scan".into(),
                1.0,
                6.0
            ),
        ]
    );
}

#[tokio::test]
async fn a_user_reference_with_a_quote_still_makes_a_valid_json_registration() {
    let (pool, vault) = connected(31_535_999).await;
    let tokens = TokenSet {
        access_token: "access-test-1".into(),
        refresh_token: String::new(),
        expires_in: 31_535_999,
        token_type: "bearer".into(),
        user_ref: "10\"579".into(),
    };
    store::save_auth(&pool, vault.as_ref(), SourceId::POLAR, &tokens)
        .await
        .unwrap();
    let fake = healthy();

    run(&pool, &vault, &fake.base_url).await.unwrap();

    let form = &fake.calls_on("/v3/users")[0].form;
    let parsed: serde_json::Value = serde_json::from_str(form).expect("JSON valide");
    assert_eq!(parsed["member-id"], "synk-10\"579");
}
