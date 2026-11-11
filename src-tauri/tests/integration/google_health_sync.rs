use crate::common::{memory_pool, test_vault};
use crate::support::fake_server::{fixture, Call, FakeServer, Reply};
use sqlx::SqlitePool;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use synk_lib::crypto::CryptoVault;
use synk_lib::error::AppError;
use synk_lib::sources::google_health::{GoogleHealthSource, OAUTH};
use synk_lib::sources::oauth::pkce::Verifier;
use synk_lib::sources::oauth::reauth::REFRESH_EXPIRY;
use synk_lib::sources::oauth::{exchange_and_store, TokenSet};
use synk_lib::sources::sync::{sync_sources, Progress};
use synk_lib::sources::{store, Domain, Source, SourceId, SyncBatch, SyncContext};

type SleepLine = (
    String,
    Option<i64>,
    Option<i64>,
    Option<i64>,
    Option<f64>,
    String,
);

const GOOD_TOKEN: &str = "Bearer access-test-2";
const DATA: &str = "/v4/users/me/dataTypes/";

fn api(call: &Call, accepted: &str) -> Reply {
    if call.authorization.as_deref() != Some(accepted) {
        return Reply::new(401, fixture("google_health", "error_401.json"));
    }
    let route = call.route();
    if route == "/v4/users/me/identity" {
        return Reply::new(200, fixture("google_health", "identity.json"));
    }
    let Some(rest) = route.strip_prefix(DATA) else {
        return Reply::new(404, "{}");
    };
    let name = match rest {
        "sleep/dataPoints" => {
            if call.query_value("pageToken").as_deref() == Some("page-2-test") {
                "sleep_page2.json"
            } else {
                "sleep_page1.json"
            }
        }
        "daily-respiratory-rate/dataPoints" => "respiratory.json",
        "exercise/dataPoints" => "exercise.json",
        "daily-resting-heart-rate/dataPoints" => "resting.json",
        "daily-heart-rate-variability/dataPoints" => "hrv.json",
        "daily-oxygen-saturation/dataPoints" => "spo2.json",
        "daily-sleep-temperature-derivations/dataPoints" => "temperature.json",
        "steps/dataPoints:dailyRollUp" => "rollup_steps.json",
        "distance/dataPoints:dailyRollUp" => "rollup_distance.json",
        "active-energy-burned/dataPoints:dailyRollUp" => "rollup_active.json",
        "total-calories/dataPoints:dailyRollUp" => "rollup_total.json",
        _ => return Reply::new(404, "{}"),
    };
    Reply::new(200, fixture("google_health", name))
}

fn healthy(accepted: &'static str) -> FakeServer {
    FakeServer::start_replying(move |call| {
        if call.route() == "/token" {
            return Reply::new(200, fixture("google_health", "token_refreshed.json"));
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
        SourceId::GOOGLE_HEALTH,
        "client-test",
        "secret-test",
    )
    .await
    .unwrap();
    let tokens = TokenSet {
        access_token: access.into(),
        refresh_token: "refresh-test-0".into(),
        expires_in,
        token_type: "Bearer".into(),
        user_ref: String::new(),
    };
    store::save_auth(&pool, vault.as_ref(), SourceId::GOOGLE_HEALTH, &tokens)
        .await
        .unwrap();
    (pool, vault)
}

async fn run(
    pool: &SqlitePool,
    vault: &Arc<CryptoVault>,
    base: &str,
) -> Result<SyncBatch, AppError> {
    let source = GoogleHealthSource::at(base);
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
            "SELECT COUNT(*) FROM sleep_records WHERE source = 'google_health'",
        )
        .await,
        count(
            pool,
            "SELECT COUNT(*) FROM activity WHERE source = 'google_health'",
        )
        .await,
        count(
            pool,
            "SELECT COUNT(*) FROM workouts WHERE source = 'google_health'",
        )
        .await,
        count(
            pool,
            "SELECT COUNT(*) FROM vitals_legacy WHERE source = 'google_health'",
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
async fn a_first_sync_checks_the_account_then_writes_every_domain_and_follows_the_page_token() {
    let (pool, vault) = connected("access-test-2", 86400).await;
    let fake = healthy(GOOD_TOKEN);

    let batch = run(&pool, &vault, &fake.base_url).await.unwrap();

    assert!(batch.errors.is_empty(), "{:?}", batch.errors);
    let sleeps: Vec<SleepLine> = sqlx::query_as(
        "SELECT date, duration_total, duration_deep, total_timeinbed, rr_average, external_id \
             FROM sleep_records WHERE source = 'google_health' ORDER BY date",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        sleeps,
        vec![
            (
                "2026-10-24".into(),
                Some(25_200),
                Some(5_400),
                Some(27_600),
                Some(14.6),
                "sleep-test-1".into()
            ),
            (
                "2026-10-25".into(),
                Some(24_000),
                Some(4_800),
                Some(27_600),
                Some(14.2),
                "sleep-test-2".into()
            ),
        ]
    );
    let activity: (i64, Option<i64>, Option<f64>, Option<f64>, Option<f64>) = sqlx::query_as(
        "SELECT COUNT(*), MAX(steps), MAX(distance), MAX(calories_active), MAX(calories_total) \
         FROM activity WHERE source = 'google_health'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        activity,
        (2, Some(9876), Some(7800.0), Some(512.5), Some(2450.2))
    );
    let workout: (String, i64, String, String) = sqlx::query_as(
        "SELECT category_label, duration, start_at, external_id FROM workouts WHERE source = 'google_health'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        workout,
        (
            "run".into(),
            2700,
            "2026-10-24T14:00:00Z".into(),
            "exercise-test-1".into()
        )
    );
    for (kind, expected) in [
        ("resting_heart_rate", 2),
        ("hrv", 2),
        ("spo2", 1),
        ("temperature_deviation", 2),
    ] {
        let n: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM vitals_legacy WHERE source = 'google_health' AND vital_type = ?1",
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

    let sleep_calls = fake.calls_on("/v4/users/me/dataTypes/sleep/dataPoints");
    assert_eq!(sleep_calls.len(), 2);
    assert_eq!(sleep_calls[0].query_value("pageToken"), None);
    assert_eq!(
        sleep_calls[0].query_value("pageSize").as_deref(),
        Some("25")
    );
    assert_eq!(
        sleep_calls[0].query_value("filter"),
        Some(format!(
            "sleep.interval.civil_end_time >= \"{}\" AND sleep.interval.civil_end_time < \"{}\"",
            day(-180),
            day(1)
        ))
    );
    assert_eq!(
        sleep_calls[1].query_value("pageToken").as_deref(),
        Some("page-2-test")
    );
    let daily = fake.calls_on("/v4/users/me/dataTypes/daily-heart-rate-variability/dataPoints");
    assert_eq!(
        daily[0].query_value("filter"),
        Some(format!(
            "daily_heart_rate_variability.date >= \"{}\" AND daily_heart_rate_variability.date < \"{}\"",
            day(-180),
            day(1)
        ))
    );
    assert!(fake.calls_on("/token").is_empty());
    let first = fake.calls_on("/v4/users/me/identity");
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].authorization.as_deref(), Some(GOOD_TOKEN));
}

#[tokio::test]
async fn rollups_are_cut_to_the_documented_ranges_and_send_a_civil_closed_open_body() {
    let (pool, vault) = connected("access-test-2", 86400).await;
    let fake = healthy(GOOD_TOKEN);

    run(&pool, &vault, &fake.base_url).await.unwrap();

    let steps = fake.calls_on("/v4/users/me/dataTypes/steps/dataPoints:dailyRollUp");
    let total = fake.calls_on("/v4/users/me/dataTypes/total-calories/dataPoints:dailyRollUp");
    assert_eq!(steps.len(), 3);
    assert_eq!(total.len(), 13);
    let body: serde_json::Value = serde_json::from_str(&steps[0].form).expect("corps JSON");
    assert_eq!(body["windowSizeDays"], 1);
    let start = &body["range"]["start"];
    let end = &body["range"]["end"];
    assert_eq!(start["time"], serde_json::json!({}));
    assert_eq!(end["time"], serde_json::json!({}));
    let text = |d: &serde_json::Value| {
        format!(
            "{:04}-{:02}-{:02}",
            d["date"]["year"].as_i64().unwrap(),
            d["date"]["month"].as_i64().unwrap(),
            d["date"]["day"].as_i64().unwrap()
        )
    };
    assert_eq!(text(start), day(-180));
    let spans: Vec<i64> = total
        .iter()
        .map(|c| {
            let b: serde_json::Value = serde_json::from_str(&c.form).unwrap();
            let parse = |d: &serde_json::Value| {
                chrono::NaiveDate::from_ymd_opt(
                    d["date"]["year"].as_i64().unwrap() as i32,
                    d["date"]["month"].as_u64().unwrap() as u32,
                    d["date"]["day"].as_u64().unwrap() as u32,
                )
                .unwrap()
            };
            (parse(&b["range"]["end"]) - parse(&b["range"]["start"])).num_days()
        })
        .collect();
    assert!(spans.iter().all(|d| (1..=14).contains(d)), "{spans:?}");
    assert_eq!(spans.iter().sum::<i64>(), 181);
}

#[tokio::test]
async fn an_incremental_sync_restarts_a_week_back_and_duplicates_nothing() {
    let (pool, vault) = connected("access-test-2", 86400).await;
    let fake = healthy(GOOD_TOKEN);
    run(&pool, &vault, &fake.base_url).await.unwrap();
    let before = totals(&pool).await;

    let second = run(&pool, &vault, &fake.base_url).await.unwrap();

    assert_eq!(before, totals(&pool).await);
    assert_eq!(second.written_in(Domain::Vitals), 0);
    let exercise = fake.calls_on("/v4/users/me/dataTypes/exercise/dataPoints");
    assert_eq!(exercise.len(), 2);
    assert_eq!(
        exercise[1].query_value("filter"),
        Some(format!(
            "exercise.interval.civil_start_time >= \"{}\" AND exercise.interval.civil_start_time < \"{}\"",
            day(-7),
            day(1)
        ))
    );
    let cursor = store::get_last_sync(&pool, SourceId::GOOGLE_HEALTH, "activity")
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
    let refreshes = fake.calls_on("/token");
    assert_eq!(refreshes.len(), 1);
    assert_eq!(
        refreshes[0].form_value("grant_type").as_deref(),
        Some("refresh_token")
    );
    assert_eq!(
        refreshes[0].form_value("refresh_token").as_deref(),
        Some("refresh-test-0")
    );
    let stored = store::load_auth(&pool, vault.as_ref(), SourceId::GOOGLE_HEALTH)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored.refresh_token.as_str(), "refresh-test-0");
    assert_eq!(stored.access_token.as_str(), "access-test-2");
}

#[tokio::test]
async fn a_token_the_server_rejects_is_refreshed_once_and_the_call_replayed() {
    let (pool, vault) = connected("access-stale", 86400).await;
    let fake = healthy(GOOD_TOKEN);

    let batch = run(&pool, &vault, &fake.base_url).await.unwrap();

    assert!(batch.errors.is_empty(), "{:?}", batch.errors);
    assert_eq!(fake.calls_on("/token").len(), 1);
    let identity = fake.calls_on("/v4/users/me/identity");
    assert_eq!(
        identity[0].authorization.as_deref(),
        Some("Bearer access-stale")
    );
    assert_eq!(identity[1].authorization.as_deref(), Some(GOOD_TOKEN));
    assert_eq!(totals(&pool).await.0, 2);
}

#[tokio::test]
async fn a_refresh_token_dead_after_seven_days_is_named_and_keeps_the_stored_tokens() {
    let (pool, vault) = connected("access-stale", 86400).await;
    let fake = FakeServer::start_replying(|call| {
        if call.route() == "/token" {
            return Reply::new(400, fixture("google_health", "token_refused.json"));
        }
        api(call, GOOD_TOKEN)
    });

    let err = run(&pool, &vault, &fake.base_url).await.unwrap_err();

    assert_eq!(err.kind(), "Unauthorized");
    let shown = err.to_string();
    assert!(shown.contains("reconnectez Google Health"), "{shown}");
    assert!(shown.contains("7 jours"), "{shown}");
    let stored = store::load_auth(&pool, vault.as_ref(), SourceId::GOOGLE_HEALTH)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored.access_token.as_str(), "access-stale");
    assert_eq!(count(&pool, "SELECT COUNT(*) FROM sleep_records").await, 0);
}

#[tokio::test]
async fn an_account_not_linked_to_google_health_stops_the_source_without_writing() {
    let (pool, vault) = connected("access-test-2", 86400).await;
    let fake = FakeServer::start_replying(|call| {
        if call.route() == "/v4/users/me/identity" {
            return Reply::new(400, fixture("google_health", "identity_not_linked.json"));
        }
        api(call, GOOD_TOKEN)
    });

    let err = run(&pool, &vault, &fake.base_url).await.unwrap_err();

    assert_eq!(err.kind(), "Unauthorized");
    assert!(err.to_string().contains("pas lie a Google Health"), "{err}");
    assert_eq!(totals(&pool).await, (0, 0, 0, 0));
    assert_eq!(
        fake.calls_on("/v4/users/me/dataTypes/sleep/dataPoints")
            .len(),
        0
    );
}

#[tokio::test]
async fn a_rate_limited_call_waits_for_retry_after_and_then_succeeds() {
    let (pool, vault) = connected("access-test-2", 86400).await;
    let hits = Arc::new(AtomicUsize::new(0));
    let seen = hits.clone();
    let fake = FakeServer::start_replying(move |call| {
        if call.route() == "/v4/users/me/dataTypes/exercise/dataPoints"
            && seen.fetch_add(1, Ordering::SeqCst) == 0
        {
            return Reply::new(429, fixture("google_health", "error_429.json"))
                .with_header("Retry-After", "0");
        }
        api(call, GOOD_TOKEN)
    });

    let batch = run(&pool, &vault, &fake.base_url).await.unwrap();

    assert!(batch.errors.is_empty(), "{:?}", batch.errors);
    assert_eq!(
        fake.calls_on("/v4/users/me/dataTypes/exercise/dataPoints")
            .len(),
        2
    );
    assert_eq!(totals(&pool).await.2, 1);
}

#[tokio::test]
async fn a_rate_limit_that_never_lifts_costs_one_step_and_spares_the_others() {
    let (pool, vault) = connected("access-test-2", 86400).await;
    let fake = FakeServer::start_replying(|call| {
        if call.route() == "/v4/users/me/dataTypes/exercise/dataPoints" {
            return Reply::new(429, fixture("google_health", "error_429.json"))
                .with_header("Retry-After", "0");
        }
        api(call, GOOD_TOKEN)
    });

    let batch = run(&pool, &vault, &fake.base_url).await.unwrap();

    assert_eq!(
        fake.calls_on("/v4/users/me/dataTypes/exercise/dataPoints")
            .len(),
        3
    );
    assert_eq!(batch.errors.len(), 1, "{:?}", batch.errors);
    assert!(
        batch.errors[0].starts_with("Entraînements : "),
        "{:?}",
        batch.errors
    );
    assert!(batch.errors[0].contains("Limite"), "{:?}", batch.errors);
    let (sleep, activity, workouts, vitals) = totals(&pool).await;
    assert_eq!((sleep, activity, workouts), (2, 2, 0));
    assert!(vitals > 0);
    let cursor = store::get_last_sync(&pool, SourceId::GOOGLE_HEALTH, "workouts")
        .await
        .unwrap();
    assert_eq!(cursor, 0);
}

#[tokio::test]
async fn a_refused_scope_is_named_and_the_body_never_reaches_the_error() {
    let (pool, vault) = connected("access-test-2", 86400).await;
    let fake = FakeServer::start_replying(|call| {
        if call.route() == "/v4/users/me/dataTypes/exercise/dataPoints" {
            return Reply::new(403, "refresh_token=RT-FUITE-9 access_token=AT-FUITE-9");
        }
        api(call, GOOD_TOKEN)
    });

    let batch = run(&pool, &vault, &fake.base_url).await.unwrap();

    assert_eq!(batch.errors.len(), 1, "{:?}", batch.errors);
    let message = &batch.errors[0];
    assert!(message.starts_with("Entraînements : "), "{message}");
    assert!(message.contains("portee"), "{message}");
    assert!(!message.contains("FUITE"), "{message}");
}

#[tokio::test]
async fn the_code_exchange_carries_pkce_and_the_exact_callback_then_the_source_syncs() {
    let pool = memory_pool().await;
    let vault = test_vault();
    store::set_credentials(
        &pool,
        vault.as_ref(),
        SourceId::GOOGLE_HEALTH,
        "client-test",
        "secret-test",
    )
    .await
    .unwrap();
    assert!(!store::is_connected(&pool, SourceId::GOOGLE_HEALTH)
        .await
        .unwrap());
    let fake = FakeServer::start_replying(|call| {
        if call.route() == "/token" {
            return Reply::new(200, fixture("google_health", "token.json"));
        }
        api(call, "Bearer access-test-1")
    });
    let verifier = Verifier::from_secret("verifier-test");
    let before = chrono::Utc::now().timestamp();

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

    let call = &fake.calls_on("/token")[0];
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
        Some("http://localhost:17483/callback/google_health")
    );
    assert_eq!(call.form_value("client_id").as_deref(), Some("client-test"));
    assert!(store::is_connected(&pool, SourceId::GOOGLE_HEALTH)
        .await
        .unwrap());
    let noted = store::get_last_sync(&pool, SourceId::GOOGLE_HEALTH, REFRESH_EXPIRY)
        .await
        .unwrap();
    assert!(noted >= before + 7 * 86_400, "{noted}");

    let source = GoogleHealthSource::at(&fake.base_url);
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
    assert_eq!(outcomes[0].label, "Google Health");
    assert_eq!(
        events.into_inner().unwrap(),
        vec![
            "google_health:1/4",
            "google_health:2/4",
            "google_health:3/4",
            "google_health:4/4",
            "analyses:1/1"
        ]
    );
}

#[tokio::test]
async fn every_metric_the_source_writes_is_already_defined_by_the_bootstrap() {
    let pool = memory_pool().await;
    synk_lib::db::bootstrap_schema(&pool).await.unwrap();
    let known: Vec<String> = sqlx::query_scalar(
        "SELECT slug FROM metric_definitions WHERE slug IN \
         ('resting_heart_rate', 'hrv', 'spo2', 'temperature_deviation') ORDER BY slug",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        known,
        vec!["hrv", "resting_heart_rate", "spo2", "temperature_deviation"]
    );
    let google = synk_lib::sources::require_source(SourceId::GOOGLE_HEALTH).unwrap();
    assert!(google.metric_definitions().is_empty());
}
