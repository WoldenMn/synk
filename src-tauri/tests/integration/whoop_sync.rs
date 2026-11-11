use crate::common::{memory_pool, test_vault};
use crate::support::fake_server::{fixture, Call, FakeServer, Reply};
use sqlx::SqlitePool;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use synk_lib::crypto::CryptoVault;
use synk_lib::error::AppError;
use synk_lib::sources::oauth::{exchange_and_store, TokenSet};
use synk_lib::sources::sync::{sync_sources, Progress};
use synk_lib::sources::whoop::{Clock, Pacer, Sleeping, WhoopSource, OAUTH, PER_MINUTE, WINDOW};
use synk_lib::sources::{store, Domain, Source, SourceId, SyncBatch, SyncContext};

type SleepLine = (String, Option<i64>, Option<i64>, Option<f64>, String);
type ActivityLine = (String, Option<f64>, Option<i64>, Option<i64>, Option<i64>);

const GOOD_TOKEN: &str = "Bearer access-test-2";
const SLEEP: &str = "/developer/v2/activity/sleep";
const RECOVERY: &str = "/developer/v2/recovery";
const CYCLE: &str = "/developer/v2/cycle";
const WORKOUT: &str = "/developer/v2/activity/workout";
const LIMIT_MINUTE: &str = "100, 100;window=60, 10000;window=86400";
const LIMIT_DAY: &str = "10000, 100;window=60, 10000;window=86400";

struct ManualClock {
    millis: AtomicU64,
    slept: Mutex<Vec<Duration>>,
}

impl ManualClock {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            millis: AtomicU64::new(0),
            slept: Mutex::new(Vec::new()),
        })
    }

    fn slept(&self) -> Vec<Duration> {
        self.slept.lock().unwrap().clone()
    }
}

impl Clock for ManualClock {
    fn now(&self) -> Duration {
        Duration::from_millis(self.millis.load(Ordering::SeqCst))
    }

    fn sleep(&self, wait: Duration) -> Sleeping<'_> {
        self.millis
            .fetch_add(wait.as_millis() as u64, Ordering::SeqCst);
        self.slept.lock().unwrap().push(wait);
        Box::pin(async {})
    }
}

fn api(call: &Call, accepted: &str) -> Reply {
    if call.authorization.as_deref() != Some(accepted) {
        return Reply::new(401, fixture("whoop", "error_401.json"));
    }
    let name = match call.route() {
        SLEEP => {
            if call.query_value("nextToken").as_deref() == Some("page-2-test") {
                "sleep_page2.json"
            } else {
                "sleep_page1.json"
            }
        }
        RECOVERY => "recovery.json",
        CYCLE => "cycle.json",
        WORKOUT => "workout.json",
        _ => return Reply::new(404, "{}"),
    };
    Reply::new(200, fixture("whoop", name))
}

fn healthy(accepted: &'static str) -> FakeServer {
    FakeServer::start_replying(move |call| {
        if call.route() == "/oauth/oauth2/token" {
            return Reply::new(200, fixture("whoop", "token_refreshed.json"));
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
        SourceId::WHOOP,
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
    store::save_auth(&pool, vault.as_ref(), SourceId::WHOOP, &tokens)
        .await
        .unwrap();
    (pool, vault)
}

async fn run_with(
    pool: &SqlitePool,
    vault: &Arc<CryptoVault>,
    base: &str,
    per_minute: usize,
    clock: Arc<ManualClock>,
) -> Result<SyncBatch, AppError> {
    let source = WhoopSource::at_paced(base, per_minute, clock);
    let ctx = SyncContext {
        pool,
        vault,
        on_progress: &|_, _| {},
    };
    source.sync(&ctx).await
}

async fn run(
    pool: &SqlitePool,
    vault: &Arc<CryptoVault>,
    base: &str,
) -> Result<SyncBatch, AppError> {
    run_with(pool, vault, base, PER_MINUTE, ManualClock::new()).await
}

async fn count(pool: &SqlitePool, sql: &'static str) -> i64 {
    sqlx::query_scalar(sql).fetch_one(pool).await.unwrap()
}

async fn totals(pool: &SqlitePool) -> (i64, i64, i64, i64) {
    (
        count(
            pool,
            "SELECT COUNT(*) FROM sleep_records WHERE source = 'whoop'",
        )
        .await,
        count(pool, "SELECT COUNT(*) FROM activity WHERE source = 'whoop'").await,
        count(pool, "SELECT COUNT(*) FROM workouts WHERE source = 'whoop'").await,
        count(
            pool,
            "SELECT COUNT(*) FROM vitals_legacy WHERE source = 'whoop'",
        )
        .await,
    )
}

fn start_param(offset_days: i64) -> String {
    (chrono::Utc::now() + chrono::Duration::days(offset_days))
        .format("%Y-%m-%dT00:00:00.000Z")
        .to_string()
}

#[tokio::test]
async fn a_first_sync_writes_every_domain_and_follows_the_next_token() {
    let (pool, vault) = connected("access-test-2", 86400).await;
    let fake = healthy(GOOD_TOKEN);

    let batch = run(&pool, &vault, &fake.base_url).await.unwrap();

    assert!(batch.errors.is_empty(), "{:?}", batch.errors);
    let sleeps: Vec<SleepLine> = sqlx::query_as(
        "SELECT date, duration_total, sleep_score, rr_average, external_id FROM sleep_records \
         WHERE source = 'whoop' ORDER BY date",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        sleeps,
        vec![
            (
                "2026-10-24".into(),
                Some(28200),
                Some(88),
                Some(15.5),
                "sl-test-1".into()
            ),
            (
                "2026-10-25".into(),
                Some(23400),
                Some(71),
                Some(16.0),
                "sl-test-2".into()
            ),
        ]
    );
    let activity: Vec<ActivityLine> = sqlx::query_as(
        "SELECT date, calories_total, hr_average, hr_max, steps FROM activity \
             WHERE source = 'whoop' ORDER BY date",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        activity,
        vec![
            ("2026-10-24".into(), Some(2250.0), Some(68), Some(160), None),
            ("2026-10-25".into(), Some(1200.0), Some(62), Some(131), None),
        ]
    );
    let workout: (String, i64, String, Option<f64>, Option<f64>, Option<i64>) = sqlx::query_as(
        "SELECT category_label, duration, start_at, distance, calories, hr_max FROM workouts \
         WHERE source = 'whoop'",
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
            Some(7200.5),
            Some(400.0),
            Some(178)
        )
    );
    for (kind, expected) in [
        ("hrv", 2),
        ("recovery_score", 2),
        ("resting_heart_rate", 2),
        ("spo2", 1),
        ("skin_temperature", 1),
        ("strain", 2),
    ] {
        let n: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM vitals_legacy WHERE source = 'whoop' AND vital_type = ?1",
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
    assert_eq!(batch.written_in(Domain::Vitals), 10);

    let sleep_calls = fake.calls_on(SLEEP);
    assert_eq!(sleep_calls.len(), 2);
    assert_eq!(sleep_calls[0].query_value("nextToken"), None);
    assert_eq!(sleep_calls[0].query_value("limit").as_deref(), Some("25"));
    assert_eq!(sleep_calls[0].query_value("start"), Some(start_param(-180)));
    assert_eq!(
        sleep_calls[1].query_value("nextToken").as_deref(),
        Some("page-2-test")
    );
    assert!(fake.calls_on("/oauth/oauth2/token").is_empty());
}

#[tokio::test]
async fn an_incremental_sync_restarts_a_week_back_and_duplicates_nothing() {
    let (pool, vault) = connected("access-test-2", 86400).await;
    let fake = healthy(GOOD_TOKEN);
    run(&pool, &vault, &fake.base_url).await.unwrap();
    let before = totals(&pool).await;

    let second = run(&pool, &vault, &fake.base_url).await.unwrap();

    assert_eq!(totals(&pool).await, before);
    assert_eq!(second.written_in(Domain::Vitals), 0);
    let cycle_calls = fake.calls_on(CYCLE);
    assert_eq!(cycle_calls.len(), 2);
    assert_eq!(cycle_calls[1].query_value("start"), Some(start_param(-7)));
    for domain in ["sleep", "cycles", "workouts"] {
        let cursor = store::get_last_sync(&pool, SourceId::WHOOP, domain)
            .await
            .unwrap();
        assert!(cursor > 0, "{domain}");
    }
}

#[tokio::test]
async fn an_expired_access_token_is_refreshed_before_the_first_call() {
    let (pool, vault) = connected("access-test-1", -100).await;
    let fake = healthy(GOOD_TOKEN);

    let batch = run(&pool, &vault, &fake.base_url).await.unwrap();

    assert!(batch.errors.is_empty(), "{:?}", batch.errors);
    let refreshes = fake.calls_on("/oauth/oauth2/token");
    assert_eq!(refreshes.len(), 1);
    assert_eq!(
        refreshes[0].form_value("grant_type").as_deref(),
        Some("refresh_token")
    );
    assert_eq!(
        refreshes[0].form_value("refresh_token").as_deref(),
        Some("refresh-test-0")
    );
    assert_eq!(
        refreshes[0].form_value("client_id").as_deref(),
        Some("client-test")
    );
    let stored = store::load_auth(&pool, vault.as_ref(), SourceId::WHOOP)
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
    assert_eq!(fake.calls_on("/oauth/oauth2/token").len(), 1);
    let first = fake.calls_on(SLEEP);
    assert_eq!(
        first[0].authorization.as_deref(),
        Some("Bearer access-stale")
    );
    assert_eq!(first[1].authorization.as_deref(), Some(GOOD_TOKEN));
    assert_eq!(totals(&pool).await.0, 2);
}

#[tokio::test]
async fn a_refused_refresh_stops_the_sync_as_unauthorized_and_keeps_the_stored_tokens() {
    let (pool, vault) = connected("access-stale", 86400).await;
    let fake = FakeServer::start_replying(|call| {
        if call.route() == "/oauth/oauth2/token" {
            return Reply::new(400, fixture("whoop", "token_refused.json"));
        }
        api(call, GOOD_TOKEN)
    });

    let err = run(&pool, &vault, &fake.base_url).await.unwrap_err();

    assert_eq!(err.kind(), "Unauthorized");
    let stored = store::load_auth(&pool, vault.as_ref(), SourceId::WHOOP)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored.access_token.as_str(), "access-stale");
    assert_eq!(totals(&pool).await, (0, 0, 0, 0));
}

#[tokio::test]
async fn a_minute_limit_is_waited_out_on_the_injected_clock_and_the_call_replayed() {
    let (pool, vault) = connected("access-test-2", 86400).await;
    let hits = Arc::new(AtomicUsize::new(0));
    let seen = hits.clone();
    let fake = FakeServer::start_replying(move |call| {
        if call.route() == RECOVERY && seen.fetch_add(1, Ordering::SeqCst) == 0 {
            return Reply::new(429, fixture("whoop", "error_429.json"))
                .with_header("X-RateLimit-Limit", LIMIT_MINUTE)
                .with_header("X-RateLimit-Remaining", "0")
                .with_header("X-RateLimit-Reset", "7");
        }
        api(call, GOOD_TOKEN)
    });
    let clock = ManualClock::new();
    let started = std::time::Instant::now();

    let batch = run_with(&pool, &vault, &fake.base_url, PER_MINUTE, clock.clone())
        .await
        .unwrap();

    assert!(batch.errors.is_empty(), "{:?}", batch.errors);
    assert_eq!(fake.calls_on(RECOVERY).len(), 2);
    assert_eq!(clock.slept(), vec![Duration::from_secs(7)]);
    assert!(started.elapsed() < Duration::from_secs(5));
    assert_eq!(totals(&pool).await.3, 10);
}

#[tokio::test]
async fn a_minute_limit_that_never_lifts_stops_the_source_without_a_retry_loop() {
    let (pool, vault) = connected("access-test-2", 86400).await;
    let fake = FakeServer::start_replying(|call| {
        if call.route() == RECOVERY {
            return Reply::new(429, fixture("whoop", "error_429.json"))
                .with_header("X-RateLimit-Limit", LIMIT_MINUTE)
                .with_header("X-RateLimit-Reset", "2");
        }
        api(call, GOOD_TOKEN)
    });
    let clock = ManualClock::new();

    let batch = run_with(&pool, &vault, &fake.base_url, PER_MINUTE, clock.clone())
        .await
        .unwrap();

    assert_eq!(fake.calls_on(RECOVERY).len(), 3);
    assert_eq!(clock.slept(), vec![Duration::from_secs(2); 2]);
    assert_eq!(batch.errors.len(), 1, "{:?}", batch.errors);
    assert!(
        batch.errors[0].starts_with("Whoop : "),
        "{:?}",
        batch.errors
    );
    assert!(
        batch.errors[0].contains("limite a la minute"),
        "{:?}",
        batch.errors
    );
    assert!(fake.calls_on(CYCLE).is_empty());
    assert!(fake.calls_on(WORKOUT).is_empty());
    assert_eq!(totals(&pool).await, (0, 0, 0, 0));
    assert_eq!(
        store::get_last_sync(&pool, SourceId::WHOOP, "sleep")
            .await
            .unwrap(),
        0
    );
}

fn daily_quota_server(hits: Arc<AtomicUsize>) -> FakeServer {
    FakeServer::start_replying(move |call| {
        if call.route() == CYCLE {
            hits.fetch_add(1, Ordering::SeqCst);
            return Reply::new(429, fixture("whoop", "error_429.json"))
                .with_header("X-RateLimit-Limit", LIMIT_DAY)
                .with_header("X-RateLimit-Remaining", "0")
                .with_header("X-RateLimit-Reset", "40000");
        }
        api(call, GOOD_TOKEN)
    })
}

#[tokio::test]
async fn the_daily_quota_stops_the_source_keeps_the_cursors_and_names_the_resume_time() {
    let (pool, vault) = connected("access-test-2", 86400).await;
    let hits = Arc::new(AtomicUsize::new(0));
    let fake = daily_quota_server(hits.clone());
    let clock = ManualClock::new();

    let batch = run_with(&pool, &vault, &fake.base_url, PER_MINUTE, clock.clone())
        .await
        .unwrap();

    assert_eq!(hits.load(Ordering::SeqCst), 1);
    assert!(clock.slept().is_empty());
    assert_eq!(batch.errors.len(), 1, "{:?}", batch.errors);
    let message = &batch.errors[0];
    assert!(
        message.starts_with("Whoop : quota du jour atteint"),
        "{message}"
    );
    assert!(message.contains("10000"), "{message}");
    assert!(message.contains("reprise vers "), "{message}");
    assert!(fake.calls_on(WORKOUT).is_empty());
    assert_eq!(totals(&pool).await, (2, 0, 0, 8));
    assert!(
        store::get_last_sync(&pool, SourceId::WHOOP, "sleep")
            .await
            .unwrap()
            > 0
    );
    for domain in ["cycles", "workouts"] {
        assert_eq!(
            store::get_last_sync(&pool, SourceId::WHOOP, domain)
                .await
                .unwrap(),
            0,
            "{domain}"
        );
    }
    let resume = store::get_last_sync(&pool, SourceId::WHOOP, "quota")
        .await
        .unwrap();
    let wait = resume - chrono::Utc::now().timestamp();
    assert!((39_000..=40_000).contains(&wait), "{wait}");
}

#[tokio::test]
async fn while_the_daily_quota_lasts_a_new_sync_does_not_touch_the_network() {
    let (pool, vault) = connected("access-test-2", 86400).await;
    let hits = Arc::new(AtomicUsize::new(0));
    let fake = daily_quota_server(hits.clone());
    run(&pool, &vault, &fake.base_url).await.unwrap();
    let routes = [SLEEP, RECOVERY, CYCLE, WORKOUT];
    let before: Vec<usize> = routes.iter().map(|r| fake.calls_on(r).len()).collect();

    let again = run(&pool, &vault, &fake.base_url).await.unwrap();

    assert_eq!(again.errors.len(), 1, "{:?}", again.errors);
    assert!(
        again.errors[0].starts_with("Whoop : quota du jour atteint"),
        "{:?}",
        again.errors
    );
    let after: Vec<usize> = routes.iter().map(|r| fake.calls_on(r).len()).collect();
    assert_eq!(before, after);
    assert_eq!(hits.load(Ordering::SeqCst), 1);
    assert!(again.written.is_empty());
}

#[tokio::test]
async fn once_the_quota_has_passed_the_next_sync_resumes_where_it_stopped() {
    let (pool, vault) = connected("access-test-2", 86400).await;
    let first_fake = daily_quota_server(Arc::new(AtomicUsize::new(0)));
    run(&pool, &vault, &first_fake.base_url).await.unwrap();
    store::set_last_sync(&pool, SourceId::WHOOP, "quota", 1)
        .await
        .unwrap();
    let healthy_fake = healthy(GOOD_TOKEN);

    let resumed = run(&pool, &vault, &healthy_fake.base_url).await.unwrap();

    assert!(resumed.errors.is_empty(), "{:?}", resumed.errors);
    assert_eq!(totals(&pool).await, (2, 2, 1, 10));
    let sleep_start = healthy_fake.calls_on(SLEEP)[0].query_value("start");
    assert_eq!(sleep_start, Some(start_param(-7)));
    let cycle_start = healthy_fake.calls_on(CYCLE)[0].query_value("start");
    assert_eq!(cycle_start, Some(start_param(-180)));
}

#[tokio::test]
async fn a_long_initial_sync_is_spread_without_ever_exceeding_the_per_minute_limit() {
    let (pool, vault) = connected("access-test-2", 86400).await;
    let clock = ManualClock::new();
    let stamps: Arc<Mutex<Vec<Duration>>> = Arc::new(Mutex::new(Vec::new()));
    let seen = stamps.clone();
    let at = clock.clone();
    let fake = FakeServer::start_replying(move |call| {
        seen.lock().unwrap().push(at.now());
        if call.route() == SLEEP {
            let page: usize = call
                .query_value("nextToken")
                .and_then(|t| t.strip_prefix('p').and_then(|n| n.parse().ok()))
                .unwrap_or(0);
            let next = if page < 249 {
                format!("\"p{}\"", page + 1)
            } else {
                "null".to_string()
            };
            return Reply::new(200, format!("{{\"records\":[],\"next_token\":{next}}}"));
        }
        api(call, GOOD_TOKEN)
    });

    let batch = run_with(&pool, &vault, &fake.base_url, PER_MINUTE, clock.clone())
        .await
        .unwrap();

    assert!(batch.errors.is_empty(), "{:?}", batch.errors);
    let stamps = stamps.lock().unwrap().clone();
    assert_eq!(stamps.len(), 250 + 1 + 1 + 1);
    for (i, from) in stamps.iter().enumerate() {
        let in_window = stamps[i..]
            .iter()
            .take_while(|t| **t - *from < WINDOW)
            .count();
        assert!(in_window <= PER_MINUTE, "{in_window} requetes en 60 s");
    }
    const { assert!(PER_MINUTE <= 100) };
    let slept: Duration = clock.slept().iter().sum();
    assert!(slept >= Duration::from_secs(120), "{slept:?}");
    assert!(slept <= Duration::from_secs(180), "{slept:?}");
}

#[tokio::test]
async fn the_pacer_lets_a_full_window_through_then_waits_for_the_oldest_to_leave_it() {
    let clock = ManualClock::new();
    let mut pacer = Pacer::new(clock.clone(), 3);
    let mut stamps = Vec::new();

    for _ in 0..7 {
        pacer.wait_turn().await;
        stamps.push(clock.now().as_secs());
    }

    assert_eq!(stamps, vec![0, 0, 0, 60, 60, 60, 120]);
    assert_eq!(clock.slept(), vec![WINDOW, WINDOW]);
}

#[tokio::test]
async fn the_pacer_waits_only_the_remaining_part_of_the_window() {
    let clock = ManualClock::new();
    let mut pacer = Pacer::new(clock.clone(), 2);
    pacer.wait_turn().await;
    clock.sleep(Duration::from_secs(20)).await;
    pacer.wait_turn().await;

    pacer.wait_turn().await;

    assert_eq!(clock.now(), Duration::from_secs(60));
    assert_eq!(
        clock.slept(),
        vec![Duration::from_secs(20), Duration::from_secs(40)]
    );
}

#[tokio::test]
async fn a_refused_scope_is_named_and_the_body_never_reaches_the_error() {
    let (pool, vault) = connected("access-test-2", 86400).await;
    let fake = FakeServer::start_replying(|call| {
        if call.route() == WORKOUT {
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
async fn a_server_error_is_retried_then_costs_one_step_without_leaking_the_body() {
    let (pool, vault) = connected("access-test-2", 86400).await;
    let fake = FakeServer::start_replying(|call| {
        if call.route() == CYCLE {
            return Reply::new(500, "access_token=AT-FUITE-5");
        }
        api(call, GOOD_TOKEN)
    });
    let clock = ManualClock::new();

    let batch = run_with(&pool, &vault, &fake.base_url, PER_MINUTE, clock.clone())
        .await
        .unwrap();

    assert_eq!(fake.calls_on(CYCLE).len(), 3);
    assert_eq!(clock.slept().len(), 2);
    assert_eq!(batch.errors.len(), 1, "{:?}", batch.errors);
    assert!(
        batch.errors[0].starts_with("Cycles : "),
        "{:?}",
        batch.errors
    );
    assert!(!batch.errors[0].contains("FUITE"), "{:?}", batch.errors);
    assert_eq!(totals(&pool).await.2, 1);
}

#[tokio::test]
async fn the_code_exchange_carries_the_exact_callback_without_pkce_then_the_source_syncs() {
    let pool = memory_pool().await;
    let vault = test_vault();
    store::set_credentials(
        &pool,
        vault.as_ref(),
        SourceId::WHOOP,
        "client-test",
        "secret-test",
    )
    .await
    .unwrap();
    assert!(!store::is_connected(&pool, SourceId::WHOOP).await.unwrap());
    let fake = FakeServer::start_replying(|call| {
        if call.route() == "/oauth/oauth2/token" {
            return Reply::new(200, fixture("whoop", "token.json"));
        }
        api(call, "Bearer access-test-1")
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

    let call = &fake.calls_on("/oauth/oauth2/token")[0];
    assert_eq!(
        call.form_value("grant_type").as_deref(),
        Some("authorization_code")
    );
    assert_eq!(call.form_value("code").as_deref(), Some("code-test"));
    assert_eq!(call.form_value("code_verifier"), None);
    assert_eq!(
        call.form_value("redirect_uri").as_deref(),
        Some("http://localhost:17483/callback/whoop")
    );
    assert_eq!(call.form_value("client_id").as_deref(), Some("client-test"));
    assert_eq!(
        call.form_value("client_secret").as_deref(),
        Some("secret-test")
    );
    assert!(store::is_connected(&pool, SourceId::WHOOP).await.unwrap());
    let stored = store::load_auth(&pool, vault.as_ref(), SourceId::WHOOP)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored.refresh_token.as_str(), "refresh-test-1");

    let source = WhoopSource::at_paced(&fake.base_url, PER_MINUTE, ManualClock::new());
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
    assert_eq!(outcomes[0].label, "Whoop");
    assert_eq!(
        events.into_inner().unwrap(),
        vec!["whoop:1/3", "whoop:2/3", "whoop:3/3", "analyses:1/1"]
    );
}

#[tokio::test]
async fn the_bootstrap_declares_the_whoop_metrics_once_even_when_replayed() {
    let pool = memory_pool().await;
    synk_lib::db::bootstrap_schema(&pool).await.unwrap();
    synk_lib::db::bootstrap_schema(&pool).await.unwrap();
    let rows: Vec<(String, String, f64, f64)> = sqlx::query_as(
        "SELECT slug, category, min_value, max_value FROM metric_definitions \
         WHERE slug IN ('recovery_score', 'resting_heart_rate', 'strain') ORDER BY slug",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        rows,
        vec![
            ("recovery_score".into(), "body_scan".into(), 0.0, 100.0),
            ("resting_heart_rate".into(), "body_scan".into(), 20.0, 200.0),
            ("strain".into(), "body_scan".into(), 0.0, 21.0),
        ]
    );
}
