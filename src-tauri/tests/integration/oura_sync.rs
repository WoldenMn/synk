use crate::common::{memory_pool, test_vault};
use crate::support::fake_server::{fixture, Call, FakeServer, Reply};
use sqlx::SqlitePool;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use synk_lib::crypto::CryptoVault;
use synk_lib::error::AppError;
use synk_lib::sources::oauth::pkce::Verifier;
use synk_lib::sources::oauth::{exchange_and_store, TokenSet};
use synk_lib::sources::oura::{OuraSource, OAUTH};
use synk_lib::sources::sync::{sync_sources, Progress};
use synk_lib::sources::{store, Domain, Source, SourceId, SyncBatch, SyncContext};

type SleepLine = (String, Option<i64>, Option<i64>, Option<i64>, String);

const GOOD_TOKEN: &str = "Bearer access-test-2";

fn api(call: &Call, accepted: &str) -> Reply {
    if call.authorization.as_deref() != Some(accepted) {
        return Reply::new(401, fixture("oura", "error_401.json"));
    }
    let name = match call.route() {
        "/v2/usercollection/daily_sleep" => {
            if call.query_value("next_token").as_deref() == Some("page-2-test") {
                "daily_sleep_page2.json"
            } else {
                "daily_sleep_page1.json"
            }
        }
        "/v2/usercollection/sleep" => "sleep.json",
        "/v2/usercollection/daily_activity" => "daily_activity.json",
        "/v2/usercollection/daily_readiness" => "daily_readiness.json",
        "/v2/usercollection/daily_spo2" => "daily_spo2.json",
        "/v2/usercollection/workout" => "workout.json",
        _ => return Reply::new(404, "{}"),
    };
    Reply::new(200, fixture("oura", name))
}

fn healthy(accepted: &'static str) -> FakeServer {
    FakeServer::start_replying(move |call| {
        if call.route() == "/oauth/token" {
            return Reply::new(200, fixture("oura", "token_refreshed.json"));
        }
        api(call, accepted)
    })
}

async fn connected(access: &str, expires_in: i64) -> (SqlitePool, Arc<CryptoVault>) {
    let pool = memory_pool().await;
    let vault = test_vault();
    store::set_credentials(
        &pool,
        vault.as_ref(),
        SourceId::OURA,
        "client-test",
        "secret-test",
    )
    .await
    .unwrap();
    let tokens = TokenSet {
        access_token: access.into(),
        refresh_token: "refresh-test-0".into(),
        expires_in,
        token_type: "bearer".into(),
        user_ref: String::new(),
    };
    store::save_auth(&pool, vault.as_ref(), SourceId::OURA, &tokens)
        .await
        .unwrap();
    (pool, vault)
}

async fn run(
    pool: &SqlitePool,
    vault: &Arc<CryptoVault>,
    base: &str,
) -> Result<SyncBatch, AppError> {
    let source = OuraSource::at(base);
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

fn day(offset: i64) -> String {
    (chrono::Utc::now() + chrono::Duration::days(offset))
        .format("%Y-%m-%d")
        .to_string()
}

#[tokio::test]
async fn a_first_sync_writes_every_domain_and_follows_the_next_token() {
    let (pool, vault) = connected("access-test-2", 86400).await;
    let fake = healthy(GOOD_TOKEN);

    let batch = run(&pool, &vault, &fake.base_url).await.unwrap();

    assert!(batch.errors.is_empty(), "{:?}", batch.errors);
    let sleeps: Vec<SleepLine> = sqlx::query_as(
        "SELECT date, duration_total, sleep_score, hr_min, external_id FROM sleep_records \
         WHERE source = 'oura' ORDER BY date",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        sleeps,
        vec![
            (
                "2026-10-24".into(),
                Some(25200),
                Some(83),
                Some(48),
                "sp-test-1".into()
            ),
            (
                "2026-10-25".into(),
                Some(22000),
                Some(71),
                Some(50),
                "sp-test-2".into()
            ),
        ]
    );
    let activity: (i64, Option<i64>, Option<f64>) = sqlx::query_as(
        "SELECT COUNT(*), MAX(steps), MAX(distance) FROM activity WHERE source = 'oura'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(activity, (2, Some(9876), Some(7800.0)));
    let workout: (String, i64, String) = sqlx::query_as(
        "SELECT category_label, duration, start_at FROM workouts WHERE source = 'oura'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        workout,
        ("running".into(), 2730, "2026-10-24T16:00:00Z".into())
    );
    for (kind, expected) in [
        ("hrv", 2),
        ("readiness_score", 2),
        ("temperature_deviation", 2),
        ("spo2", 1),
    ] {
        let n: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM vitals_legacy WHERE source = 'oura' AND vital_type = ?1",
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
    assert_eq!(batch.written_in(Domain::Vitals), 7);

    let sleep_calls = fake.calls_on("/v2/usercollection/daily_sleep");
    assert_eq!(sleep_calls.len(), 2);
    assert_eq!(sleep_calls[0].query_value("next_token"), None);
    assert_eq!(sleep_calls[0].query_value("start_date"), Some(day(-180)));
    assert_eq!(sleep_calls[0].query_value("end_date"), Some(day(1)));
    assert_eq!(
        sleep_calls[1].query_value("next_token").as_deref(),
        Some("page-2-test")
    );
    assert!(fake.calls_on("/oauth/token").is_empty());
}

#[tokio::test]
async fn an_incremental_sync_restarts_a_week_back_and_duplicates_nothing() {
    let (pool, vault) = connected("access-test-2", 86400).await;
    let fake = healthy(GOOD_TOKEN);
    run(&pool, &vault, &fake.base_url).await.unwrap();
    let before = (
        count(
            &pool,
            "SELECT COUNT(*) FROM sleep_records WHERE source = 'oura'",
        )
        .await,
        count(&pool, "SELECT COUNT(*) FROM activity WHERE source = 'oura'").await,
        count(&pool, "SELECT COUNT(*) FROM workouts WHERE source = 'oura'").await,
        count(
            &pool,
            "SELECT COUNT(*) FROM vitals_legacy WHERE source = 'oura'",
        )
        .await,
    );

    let second = run(&pool, &vault, &fake.base_url).await.unwrap();

    let after = (
        count(
            &pool,
            "SELECT COUNT(*) FROM sleep_records WHERE source = 'oura'",
        )
        .await,
        count(&pool, "SELECT COUNT(*) FROM activity WHERE source = 'oura'").await,
        count(&pool, "SELECT COUNT(*) FROM workouts WHERE source = 'oura'").await,
        count(
            &pool,
            "SELECT COUNT(*) FROM vitals_legacy WHERE source = 'oura'",
        )
        .await,
    );
    assert_eq!(before, after);
    assert_eq!(second.written_in(Domain::Vitals), 0);
    let activity_calls = fake.calls_on("/v2/usercollection/daily_activity");
    assert_eq!(activity_calls.len(), 2);
    assert_eq!(activity_calls[1].query_value("start_date"), Some(day(-7)));
    let cursor = store::get_last_sync(&pool, SourceId::OURA, "activity")
        .await
        .unwrap();
    assert!(cursor > 0);
}

#[tokio::test]
async fn an_expired_access_token_is_refreshed_before_the_first_call() {
    let (pool, vault) = connected("access-test-1", -100).await;
    let fake = healthy(GOOD_TOKEN);

    let batch = run(&pool, &vault, &fake.base_url).await.unwrap();

    assert!(batch.errors.is_empty(), "{:?}", batch.errors);
    let refreshes = fake.calls_on("/oauth/token");
    assert_eq!(refreshes.len(), 1);
    assert_eq!(
        refreshes[0].form_value("grant_type").as_deref(),
        Some("refresh_token")
    );
    assert_eq!(
        refreshes[0].form_value("refresh_token").as_deref(),
        Some("refresh-test-0")
    );
    let stored = store::load_auth(&pool, vault.as_ref(), SourceId::OURA)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored.refresh_token.as_str(), "refresh-test-2");
}

#[tokio::test]
async fn a_token_the_server_rejects_is_refreshed_once_and_the_call_replayed() {
    let (pool, vault) = connected("access-stale", 86400).await;
    let fake = healthy(GOOD_TOKEN);

    let batch = run(&pool, &vault, &fake.base_url).await.unwrap();

    assert!(batch.errors.is_empty(), "{:?}", batch.errors);
    assert_eq!(fake.calls_on("/oauth/token").len(), 1);
    let first_sleep = fake.calls_on("/v2/usercollection/sleep");
    assert_eq!(
        first_sleep[0].authorization.as_deref(),
        Some("Bearer access-stale")
    );
    assert_eq!(first_sleep[1].authorization.as_deref(), Some(GOOD_TOKEN));
    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM sleep_records WHERE source = 'oura'"
        )
        .await,
        2
    );
}

#[tokio::test]
async fn a_refused_refresh_stops_the_sync_as_unauthorized_and_keeps_the_stored_tokens() {
    let (pool, vault) = connected("access-stale", 86400).await;
    let fake = FakeServer::start_replying(|call| {
        if call.route() == "/oauth/token" {
            return Reply::new(400, fixture("oura", "token_refused.json"));
        }
        api(call, GOOD_TOKEN)
    });

    let err = run(&pool, &vault, &fake.base_url).await.unwrap_err();

    assert_eq!(err.kind(), "Unauthorized");
    let stored = store::load_auth(&pool, vault.as_ref(), SourceId::OURA)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored.access_token.as_str(), "access-stale");
    assert_eq!(count(&pool, "SELECT COUNT(*) FROM sleep_records").await, 0);
}

#[tokio::test]
async fn a_rate_limited_call_waits_for_retry_after_and_then_succeeds() {
    let (pool, vault) = connected("access-test-2", 86400).await;
    let hits = Arc::new(AtomicUsize::new(0));
    let seen = hits.clone();
    let fake = FakeServer::start_replying(move |call| {
        if call.route() == "/v2/usercollection/daily_activity"
            && seen.fetch_add(1, Ordering::SeqCst) == 0
        {
            return Reply::new(429, fixture("oura", "error_429.json"))
                .with_header("Retry-After", "0");
        }
        api(call, GOOD_TOKEN)
    });

    let batch = run(&pool, &vault, &fake.base_url).await.unwrap();

    assert!(batch.errors.is_empty(), "{:?}", batch.errors);
    assert_eq!(fake.calls_on("/v2/usercollection/daily_activity").len(), 2);
    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM activity WHERE source = 'oura'").await,
        2
    );
}

#[tokio::test]
async fn a_rate_limit_that_never_lifts_costs_one_step_and_spares_the_others() {
    let (pool, vault) = connected("access-test-2", 86400).await;
    let fake = FakeServer::start_replying(|call| {
        if call.route() == "/v2/usercollection/daily_activity" {
            return Reply::new(429, fixture("oura", "error_429.json"))
                .with_header("Retry-After", "0");
        }
        api(call, GOOD_TOKEN)
    });

    let batch = run(&pool, &vault, &fake.base_url).await.unwrap();

    assert_eq!(fake.calls_on("/v2/usercollection/daily_activity").len(), 3);
    assert_eq!(batch.errors.len(), 1, "{:?}", batch.errors);
    assert!(
        batch.errors[0].starts_with("Activité : "),
        "{:?}",
        batch.errors
    );
    assert!(batch.errors[0].contains("Limite"), "{:?}", batch.errors);
    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM activity WHERE source = 'oura'").await,
        0
    );
    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM sleep_records WHERE source = 'oura'"
        )
        .await,
        2
    );
    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM workouts WHERE source = 'oura'").await,
        1
    );
    let cursor = store::get_last_sync(&pool, SourceId::OURA, "activity")
        .await
        .unwrap();
    assert_eq!(cursor, 0);
}

#[tokio::test]
async fn an_expired_subscription_is_named_and_the_body_never_reaches_the_error() {
    let (pool, vault) = connected("access-test-2", 86400).await;
    let fake = FakeServer::start_replying(|call| {
        if call.route() == "/v2/usercollection/workout" {
            return Reply::new(403, fixture("oura", "error_403.json"));
        }
        api(call, GOOD_TOKEN)
    });

    let batch = run(&pool, &vault, &fake.base_url).await.unwrap();

    assert_eq!(batch.errors.len(), 1, "{:?}", batch.errors);
    let message = &batch.errors[0];
    assert!(message.starts_with("Entraînements : "), "{message}");
    assert!(message.contains("abonnement"), "{message}");
    assert!(!message.contains("FUITE"), "{message}");
}

#[tokio::test]
async fn the_code_exchange_carries_pkce_and_the_exact_callback_then_the_source_syncs() {
    let pool = memory_pool().await;
    let vault = test_vault();
    store::set_credentials(
        &pool,
        vault.as_ref(),
        SourceId::OURA,
        "client-test",
        "secret-test",
    )
    .await
    .unwrap();
    assert!(!store::is_connected(&pool, SourceId::OURA).await.unwrap());
    let fake = FakeServer::start_replying(|call| {
        if call.route() == "/oauth/token" {
            return Reply::new(200, fixture("oura", "token.json"));
        }
        api(call, "Bearer access-test-1")
    });
    let verifier = Verifier::from_secret("verifier-test");

    exchange_and_store(
        &OAUTH,
        &fake.base_url,
        &pool,
        vault.as_ref(),
        "code-test",
        Some(&verifier),
    )
    .await
    .unwrap();

    let call = &fake.calls_on("/oauth/token")[0];
    assert_eq!(
        call.form_value("grant_type").as_deref(),
        Some("authorization_code")
    );
    assert_eq!(call.form_value("code").as_deref(), Some("code-test"));
    assert_eq!(
        call.form_value("code_verifier").as_deref(),
        Some("verifier-test")
    );
    assert_eq!(
        call.form_value("redirect_uri").as_deref(),
        Some("http://localhost:17483/callback/oura")
    );
    assert_eq!(call.form_value("client_id").as_deref(), Some("client-test"));
    assert!(store::is_connected(&pool, SourceId::OURA).await.unwrap());

    let source = OuraSource::at(&fake.base_url);
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
    assert_eq!(outcomes[0].label, "Oura");
    let events = events.into_inner().unwrap();
    assert_eq!(
        events,
        vec![
            "oura:1/4",
            "oura:2/4",
            "oura:3/4",
            "oura:4/4",
            "analyses:1/1"
        ]
    );
}

#[tokio::test]
async fn the_bootstrap_declares_the_oura_metrics_once_even_when_replayed() {
    let pool = memory_pool().await;
    synk_lib::db::bootstrap_schema(&pool).await.unwrap();
    let rows: Vec<(String, String, f64, f64)> = sqlx::query_as(
        "SELECT slug, category, min_value, max_value FROM metric_definitions WHERE slug IN ('hrv', 'readiness_score', 'temperature_deviation') ORDER BY slug",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        rows,
        vec![
            ("hrv".into(), "body_scan".into(), 1.0, 400.0),
            ("readiness_score".into(), "body_scan".into(), 0.0, 100.0),
            (
                "temperature_deviation".into(),
                "body_scan".into(),
                -10.0,
                10.0
            ),
        ]
    );
}

#[tokio::test]
async fn a_value_revised_by_oura_inside_the_replay_window_replaces_the_stored_one() {
    let (pool, vault) = connected("access-test-2", 86400).await;
    let fake = healthy(GOOD_TOKEN);
    run(&pool, &vault, &fake.base_url).await.unwrap();
    let score = |pool: &SqlitePool| {
        let pool = pool.clone();
        async move {
            sqlx::query_as::<_, (f64, String)>(
                "SELECT value, external_id FROM vitals_legacy \
                 WHERE source = 'oura' AND vital_type = 'readiness_score' AND measured_at LIKE '2026-10-25%'",
            )
            .fetch_one(&pool)
            .await
            .unwrap()
        }
    };
    assert_eq!(score(&pool).await.0, 85.0);

    let revised = FakeServer::start_replying(move |call| {
        if call.route() == "/v2/usercollection/daily_readiness" {
            let body =
                fixture("oura", "daily_readiness.json").replace("\"score\":85", "\"score\":91");
            return Reply::new(200, body);
        }
        api(call, GOOD_TOKEN)
    });
    let second = run(&pool, &vault, &revised.base_url).await.unwrap();

    let (value, _) = score(&pool).await;
    assert_eq!(value, 91.0);
    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM vitals_legacy WHERE source = 'oura' AND vital_type = 'readiness_score'"
        )
        .await,
        2
    );
    assert_eq!(second.written_in(Domain::Vitals), 1);

    let third = run(&pool, &vault, &revised.base_url).await.unwrap();
    assert_eq!(third.written_in(Domain::Vitals), 0);
}
