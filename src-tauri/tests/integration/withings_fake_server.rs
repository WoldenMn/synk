use crate::support::fake_server::{fixture, Call, FakeServer};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use std::sync::{Arc, Mutex};
use synk_lib::crypto::CryptoVault;
use synk_lib::sources::withings::client::{save_tokens, WithingsClient};
use synk_lib::sources::withings::types::{SleepBody, TokenBody};
use zeroize::Zeroizing;

fn withings(name: &str) -> String {
    fixture("withings", name)
}

async fn setup(expires_in: i64) -> (SqlitePool, Arc<CryptoVault>) {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(":memory:")
        .await
        .unwrap();
    synk_lib::db::bootstrap_schema(&pool).await.unwrap();
    let vault = crate::common::test_vault();
    let tokens = TokenBody {
        access_token: "AT-1".into(),
        refresh_token: "RT-1".into(),
        expires_in,
        token_type: "Bearer".into(),
        userid: serde_json::Value::String("4242".into()),
    };
    save_tokens(&pool, &tokens, vault.as_ref()).await.unwrap();
    crate::common::set_pair(&pool, vault.as_ref(), "cid", "csecret")
        .await
        .unwrap();
    (pool, vault)
}

fn client(fake: &FakeServer, pool: &SqlitePool, vault: &Arc<CryptoVault>) -> WithingsClient {
    WithingsClient::at(&fake.base_url, Zeroizing::new("AT-1".into()))
        .unwrap()
        .with_pool(pool.clone())
        .with_vault(vault.clone())
}

#[tokio::test]
async fn the_fake_server_answers_and_records() {
    let (pool, vault) = setup(10_800).await;
    let fake = FakeServer::start(|_: &Call| (200, withings("sleep_getsummary.json")));

    let resp = client(&fake, &pool, &vault)
        .post::<SleepBody>("/v2/sleep", &[("action", "getsummary")])
        .await
        .unwrap();

    assert_eq!(resp.body.series.len(), 2);
    let calls = fake.calls_to("/v2/sleep");
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].authorization.as_deref(), Some("Bearer AT-1"));
    assert!(
        calls[0].form.contains("action=getsummary"),
        "{}",
        calls[0].form
    );
}

fn sleep_after_refresh(call: &Call) -> (u16, String) {
    match (call.path.as_str(), call.authorization.as_deref()) {
        ("/v2/oauth2", _) => (200, withings("token_refresh.json")),
        ("/v2/sleep", Some("Bearer AT-2")) => (200, withings("sleep_getsummary.json")),
        ("/v2/sleep", _) => (200, withings("envelope_401.json")),
        _ => (404, String::new()),
    }
}

#[tokio::test]
async fn envelope_401_triggers_one_refresh_then_succeeds() {
    let (pool, vault) = setup(10_800).await;
    let fake = FakeServer::start(sleep_after_refresh);

    let resp = client(&fake, &pool, &vault)
        .post::<SleepBody>("/v2/sleep", &[("action", "getsummary")])
        .await
        .unwrap();

    assert_eq!(resp.body.series.len(), 2);
    let refresh = fake.calls_to("/v2/oauth2");
    assert_eq!(refresh.len(), 1);
    assert!(
        refresh[0].form.contains("grant_type=refresh_token"),
        "{}",
        refresh[0].form
    );
    assert!(
        refresh[0].form.contains("refresh_token=RT-1"),
        "{}",
        refresh[0].form
    );
    let stored: String = sqlx::query_scalar("SELECT refresh_token FROM withings_auth WHERE id = 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(vault.open_str(&stored).unwrap().as_str(), "RT-2");
}

#[tokio::test]
async fn two_concurrent_refusals_share_one_refresh() {
    let (pool, vault) = setup(10_800).await;
    let fake = FakeServer::start(sleep_after_refresh);
    let c = client(&fake, &pool, &vault);

    let (a, b) = tokio::join!(
        c.post::<SleepBody>("/v2/sleep", &[("action", "getsummary")]),
        c.post::<SleepBody>("/v2/sleep", &[("action", "getsummary")])
    );

    assert!(a.is_ok() && b.is_ok(), "{a:?} {b:?}");
    assert_eq!(fake.calls_to("/v2/oauth2").len(), 1);
}

#[tokio::test]
async fn a_token_refused_after_refresh_stops_without_looping() {
    let (pool, vault) = setup(10_800).await;
    let fake = FakeServer::start(|call| match call.path.as_str() {
        "/v2/oauth2" => (200, withings("token_refresh.json")),
        _ => (200, withings("envelope_401.json")),
    });

    let err = client(&fake, &pool, &vault)
        .post::<SleepBody>("/v2/sleep", &[("action", "getsummary")])
        .await
        .unwrap_err();

    assert_eq!(err.kind(), "Unauthorized", "{err}");
    assert_eq!(fake.calls_to("/v2/oauth2").len(), 1);
    assert_eq!(fake.calls_to("/v2/sleep").len(), 2);
}

#[tokio::test]
async fn a_refused_refresh_reports_the_withings_code() {
    let (pool, vault) = setup(10_800).await;
    let fake = FakeServer::start(|call| match call.path.as_str() {
        "/v2/oauth2" => (200, withings("refresh_refused.json")),
        _ => (200, withings("envelope_401.json")),
    });

    let err = client(&fake, &pool, &vault)
        .post::<SleepBody>("/v2/sleep", &[("action", "getsummary")])
        .await
        .unwrap_err();

    assert_eq!(err.kind(), "Unauthorized", "{err}");
    assert!(err.to_string().contains("Withings code 401"), "{err}");
    assert_eq!(fake.calls_to("/v2/oauth2").len(), 1);
}

#[tokio::test]
async fn invalid_params_are_not_retried() {
    let (pool, vault) = setup(10_800).await;
    let fake = FakeServer::start(|_| (200, withings("envelope_503.json")));

    let err = client(&fake, &pool, &vault)
        .post::<SleepBody>("/v2/sleep", &[("action", "getsummary")])
        .await
        .unwrap_err();

    assert!(
        err.to_string().contains("503") && err.to_string().contains("parametres"),
        "{err}"
    );
    assert_eq!(fake.calls_to("/v2/sleep").len(), 1);
}

#[tokio::test]
async fn rate_limit_is_retried_then_succeeds() {
    let (pool, vault) = setup(10_800).await;
    let answered = Arc::new(Mutex::new(0u32));
    let count = answered.clone();
    let fake = FakeServer::start(move |_| {
        let mut n = count.lock().unwrap();
        *n += 1;
        if *n == 1 {
            (200, withings("envelope_601.json"))
        } else {
            (200, withings("sleep_getsummary.json"))
        }
    });

    let resp = client(&fake, &pool, &vault)
        .post::<SleepBody>("/v2/sleep", &[("action", "getsummary")])
        .await
        .unwrap();

    assert_eq!(resp.body.series.len(), 2);
    assert_eq!(fake.calls_to("/v2/sleep").len(), 2);
}

#[tokio::test]
async fn a_rate_limit_that_outlasts_every_retry_keeps_its_type() {
    let (pool, vault) = setup(10_800).await;
    let fake = FakeServer::start(|_| (200, withings("envelope_601.json")));

    let err = client(&fake, &pool, &vault)
        .post::<SleepBody>("/v2/sleep", &[("action", "getsummary")])
        .await
        .unwrap_err();

    assert_eq!(err.kind(), "RateLimit", "{err}");
    assert!(err.detail().contains("retries failed"), "{err}");
}

#[tokio::test]
async fn an_unreadable_token_store_is_reported_not_hidden() {
    let (pool, vault) = setup(10_800).await;
    let fake = FakeServer::start(sleep_after_refresh);
    let c = client(&fake, &pool, &vault);
    sqlx::query("ALTER TABLE source_auth RENAME TO source_auth_hors_service")
        .execute(&pool)
        .await
        .unwrap();

    let err = c
        .post::<SleepBody>("/v2/sleep", &[("action", "getsummary")])
        .await
        .unwrap_err();

    assert!(
        err.to_string().contains("rafraichissement impossible"),
        "{err}"
    );
    assert!(err.to_string().contains("source_auth"), "{err}");
}

#[tokio::test]
async fn an_expired_token_is_refreshed_once_even_by_two_callers() {
    let (pool, vault) = setup(-100).await;
    let fake = FakeServer::start(sleep_after_refresh);

    let (a, b) = tokio::join!(
        synk_lib::sources::withings::client::get_valid_client_at(&fake.base_url, &pool, &vault),
        synk_lib::sources::withings::client::get_valid_client_at(&fake.base_url, &pool, &vault)
    );

    assert!(a.is_ok() && b.is_ok());
    assert_eq!(fake.calls_to("/v2/oauth2").len(), 1);
    a.unwrap()
        .post::<SleepBody>("/v2/sleep", &[("action", "getsummary")])
        .await
        .unwrap();
    assert_eq!(
        fake.calls_to("/v2/sleep")[0].authorization.as_deref(),
        Some("Bearer AT-2")
    );
}

use synk_lib::sources::withings::sync::{
    run_sync_all, sync_activity, sync_body_measures, sync_devices, sync_heart, sync_sleep,
    sync_workouts,
};

fn every_endpoint(call: &Call) -> (u16, String) {
    let action = call.form_value("action").unwrap_or_default();
    let meastypes = call.form_value("meastypes").unwrap_or_default();
    let body = match (call.path.as_str(), action.as_str()) {
        ("/v2/sleep", "getsummary") => withings("sleep_getsummary.json"),
        ("/v2/measure", "getactivity") => withings("activity_getactivity.json"),
        ("/v2/measure", "getworkouts") => withings("workouts_getworkouts.json"),
        ("/v2/heart", "list") => withings("heart_list.json"),
        ("/v2/user", "getdevice") => withings("user_getdevice.json"),
        ("/measure", "getmeas") if meastypes.starts_with("1,") => {
            withings("measure_getmeas_body.json")
        }
        ("/measure", "getmeas") if meastypes.starts_with("9,") => {
            withings("measure_getmeas_bp.json")
        }
        ("/measure", "getmeas") => withings("measure_getmeas_empty.json"),
        _ => return (404, String::new()),
    };
    (200, body)
}

async fn scalar_i64(pool: &SqlitePool, sql: &'static str) -> i64 {
    sqlx::query_scalar(sql).fetch_one(pool).await.unwrap()
}

#[tokio::test]
async fn sleep_sync_writes_nights_without_a_metrics_copy() {
    let (pool, vault) = setup(10_800).await;
    let fake = FakeServer::start(every_endpoint);

    let n = sync_sleep(&client(&fake, &pool, &vault), &pool)
        .await
        .unwrap();

    assert_eq!(n, 2);
    assert_eq!(
        scalar_i64(
            &pool,
            "SELECT sleep_score FROM sleep_records WHERE date = '2026-09-14'"
        )
        .await,
        81
    );
    assert_eq!(
        scalar_i64(
            &pool,
            "SELECT duration_total FROM sleep_records WHERE date = '2026-09-14'"
        )
        .await,
        25200
    );
    assert_eq!(
        scalar_i64(
            &pool,
            "SELECT COUNT(*) FROM metrics m JOIN metric_definitions d ON d.id = m.definition_id WHERE d.slug = 'sleep_score'"
        )
        .await,
        0
    );
}

#[tokio::test]
async fn activity_sync_writes_the_day_without_a_metrics_copy() {
    let (pool, vault) = setup(10_800).await;
    let fake = FakeServer::start(every_endpoint);

    sync_activity(&client(&fake, &pool, &vault), &pool)
        .await
        .unwrap();

    assert_eq!(
        scalar_i64(
            &pool,
            "SELECT steps FROM activity WHERE date = '2026-09-14' AND source = 'withings'"
        )
        .await,
        8450
    );
    assert_eq!(
        scalar_i64(
            &pool,
            "SELECT COUNT(*) FROM metrics m JOIN metric_definitions d ON d.id = m.definition_id WHERE d.slug = 'steps'"
        )
        .await,
        0
    );
}

#[tokio::test]
async fn workout_sync_keeps_dated_entries_only() {
    let (pool, vault) = setup(10_800).await;
    let fake = FakeServer::start(every_endpoint);

    let n = sync_workouts(&client(&fake, &pool, &vault), &pool)
        .await
        .unwrap();

    assert_eq!(n, 1);
    assert_eq!(
        scalar_i64(
            &pool,
            "SELECT duration FROM workouts WHERE withings_id = '987'"
        )
        .await,
        3600
    );
}

#[tokio::test]
async fn heart_sync_writes_the_ecg() {
    let (pool, vault) = setup(10_800).await;
    let fake = FakeServer::start(every_endpoint);

    sync_heart(&client(&fake, &pool, &vault), &pool)
        .await
        .unwrap();

    let row: (Option<i64>, Option<i64>, Option<i64>, Option<String>) = sqlx::query_as(
        "SELECT ecg_status, heart_rate, qrs_duration, withings_ecg_id FROM heart_records",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        row,
        (Some(0), Some(64), Some(92), Some("123456".to_string()))
    );
}

#[tokio::test]
async fn device_sync_ignores_a_device_without_id() {
    let (pool, vault) = setup(10_800).await;
    let fake = FakeServer::start(every_endpoint);

    let n = sync_devices(&client(&fake, &pool, &vault), &pool)
        .await
        .unwrap();

    assert_eq!(n, 1);
    assert_eq!(
        scalar_i64(
            &pool,
            "SELECT COUNT(*) FROM withings_devices WHERE device_id = 'abc123'"
        )
        .await,
        1
    );
}

#[tokio::test]
async fn measures_sync_writes_body_and_blood_pressure_groups() {
    let (pool, vault) = setup(10_800).await;
    let fake = FakeServer::start(every_endpoint);

    let (body, bp, vitals) = sync_body_measures(&client(&fake, &pool, &vault), &pool)
        .await
        .unwrap();

    assert_eq!((body, bp, vitals), (1, 1, 0));
    let weight: f64 =
        sqlx::query_scalar("SELECT weight FROM body_measures WHERE withings_grpid = '5001'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!((weight - 72.5).abs() < 1e-6, "{weight}");
    let pressure: (i64, i64, Option<i64>) = sqlx::query_as(
        "SELECT systolic, diastolic, heart_rate FROM blood_pressure WHERE withings_grpid = '5002'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(pressure, (122, 78, Some(64)));
    assert_eq!(fake.calls_to("/measure").len(), 3);
}

async fn seed_a_weight_trend(pool: &SqlitePool) {
    for i in 0..8 {
        sqlx::query("INSERT INTO body_measures_legacy (measured_at, source, weight) VALUES (strftime('%Y-%m-%dT08:00:00Z', 'now', ?1), 'manual', ?2)")
            .bind(format!("-{} days", 20 - i * 2))
            .bind(70.0 + f64::from(i))
            .execute(pool)
            .await
            .unwrap();
    }
}

#[tokio::test]
async fn full_sync_aggregates_module_errors_and_logs_each_module() {
    let (pool, vault) = setup(10_800).await;
    seed_a_weight_trend(&pool).await;
    let fake = FakeServer::start(|call| {
        if call.path == "/v2/heart" {
            (
                200,
                r#"{ "status": 2555, "body": {}, "error": "An unknown error occurred" }"#
                    .to_string(),
            )
        } else {
            every_endpoint(call)
        }
    });
    let steps = Mutex::new(Vec::new());

    let result = run_sync_all(&pool, &client(&fake, &pool, &vault), &|n, _| {
        steps.lock().unwrap().push(n)
    })
    .await;

    assert_eq!(result.sleep_count, 2);
    assert_eq!(result.body_measures_count, 1);
    assert_eq!(result.heart_count, 0);
    assert_eq!(result.errors.len(), 1, "{:?}", result.errors);
    assert!(result.errors[0].contains("2555"), "{:?}", result.errors);
    assert_eq!(*steps.lock().unwrap(), vec![1, 2, 3, 4, 5, 6]);
    assert_eq!(
        scalar_i64(&pool, "SELECT COUNT(*) FROM insights").await,
        0,
        "les analyses se generent apres toutes les sources, plus dans la synchro Withings"
    );
    assert_eq!(
        scalar_i64(
            &pool,
            "SELECT COUNT(*) FROM sync_log WHERE status = 'error'"
        )
        .await,
        1
    );
    assert_eq!(
        scalar_i64(
            &pool,
            "SELECT COUNT(*) FROM sync_log WHERE status = 'success'"
        )
        .await,
        5
    );
}

#[tokio::test]
async fn a_malformed_token_response_never_leaks_into_the_error() {
    let (pool, vault) = setup(-100).await;
    let fake = FakeServer::start(|_: &Call| {
        (
            200,
            r#"{ "status": 0, "body": { "access_token": "AT-FUITE-1", "refresh_token": 42 } }"#
                .to_string(),
        )
    });

    let err = synk_lib::sources::withings::client::refresh_token_at(
        &fake.base_url,
        &pool,
        "RT-ANCIEN",
        &vault,
    )
    .await
    .unwrap_err();

    assert!(
        !err.to_string().contains("AT-FUITE-1"),
        "jeton dans l'erreur : {err}"
    );
    assert!(err.to_string().contains("Token JSON parse error"), "{err}");
}

#[tokio::test]
async fn an_unreadable_token_response_never_leaks_into_the_error() {
    let (pool, vault) = setup(-100).await;
    let fake = FakeServer::start(|_: &Call| {
        (
            200,
            "access_token=AT-FUITE-2&refresh_token=RT-FUITE-2".to_string(),
        )
    });

    let err = synk_lib::sources::withings::client::refresh_token_at(
        &fake.base_url,
        &pool,
        "RT-ANCIEN",
        &vault,
    )
    .await
    .unwrap_err();

    assert!(
        !err.to_string().contains("FUITE"),
        "jeton dans l'erreur : {err}"
    );
}

#[tokio::test]
async fn the_source_trait_runs_the_same_full_sync() {
    use synk_lib::sources::withings::WithingsSource;
    use synk_lib::sources::{Domain, Source, SyncContext};

    let (pool, vault) = setup(10_800).await;
    let fake = FakeServer::start(|call| {
        if call.path == "/v2/heart" {
            (
                200,
                r#"{ "status": 2555, "body": {}, "error": "An unknown error occurred" }"#
                    .to_string(),
            )
        } else {
            every_endpoint(call)
        }
    });
    let steps = Mutex::new(Vec::new());
    let source = WithingsSource::at(fake.base_url.clone());
    let source: &dyn Source = &source;
    let on_progress = |n: u32, _: &'static str| steps.lock().unwrap().push(n);
    let ctx = SyncContext {
        pool: &pool,
        vault: &vault,
        on_progress: &on_progress,
    };

    let batch = source.sync(&ctx).await.unwrap();

    assert_eq!(batch.written_in(Domain::Sleep), 2);
    assert_eq!(batch.written_in(Domain::Body), 1);
    assert_eq!(batch.written_in(Domain::Heart), 0);
    assert_eq!(batch.errors.len(), 1, "{:?}", batch.errors);
    assert!(batch.errors[0].contains("2555"), "{:?}", batch.errors);
    assert_eq!(*steps.lock().unwrap(), vec![1, 2, 3, 4, 5, 6]);
    assert_eq!(
        scalar_i64(
            &pool,
            "SELECT COUNT(*) FROM sync_log WHERE status = 'success'"
        )
        .await,
        5
    );
}

#[tokio::test]
async fn the_source_reports_a_missing_connection_as_an_error() {
    use synk_lib::sources::withings::WithingsSource;
    use synk_lib::sources::{Source, SyncContext};

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(":memory:")
        .await
        .unwrap();
    synk_lib::db::bootstrap_schema(&pool).await.unwrap();
    let vault = crate::common::test_vault();
    let source = WithingsSource::at("http://127.0.0.1:1");
    let ctx = SyncContext {
        pool: &pool,
        vault: &vault,
        on_progress: &|_, _| {},
    };

    assert!(source.sync(&ctx).await.is_err());
}
