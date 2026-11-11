use crate::common::{memory_pool_with_source, test_vault};
use crate::support::fake_server::{Call, FakeServer};
use sqlx::SqlitePool;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, Once};
use std::time::{Duration, Instant};
use synk_lib::crypto::CryptoVault;
use synk_lib::sources::oauth::flow::OAuthFlow;
use synk_lib::sources::oauth::pkce::Verifier;
use synk_lib::sources::oauth::server::{serve, CallbackSink};
use synk_lib::sources::oauth::token::{
    exchange_code, refresh_token, request_token_with, standard_json,
};
use synk_lib::sources::oauth::{exchange_and_store, refresh_locked, ClientAuth, OAuthSpec};
use synk_lib::sources::{store, SourceId};

static ACME: OAuthSpec = OAuthSpec {
    id: SourceId::new("acme"),
    authorize_url: "https://acme.test/authorize",
    api_base: "",
    token_path: "/token",
    scopes: "read write",
    pkce: true,
    redirect_path: None,
    token_params: &[],
    authorize_params: &[],
    refresh_lifetime_secs: None,
    parse_token: standard_json,
    client_auth: ClientAuth::Body,
};

static BASIC: OAuthSpec = OAuthSpec {
    id: SourceId::new("acme"),
    authorize_url: "https://acme.test/authorize",
    api_base: "",
    token_path: "/token",
    scopes: "read",
    pkce: false,
    redirect_path: None,
    token_params: &[],
    authorize_params: &[],
    refresh_lifetime_secs: None,
    parse_token: standard_json,
    client_auth: ClientAuth::Basic,
};

const FRESH: &str =
    r#"{"access_token":"AT-1","refresh_token":"RT-1","expires_in":3600,"token_type":"Bearer"}"#;

async fn setup() -> (SqlitePool, Arc<CryptoVault>) {
    let pool = memory_pool_with_source("acme").await;
    let vault = test_vault();
    store::set_credentials(&pool, vault.as_ref(), ACME.id, "cid", "csecret")
        .await
        .unwrap();
    let tokens = synk_lib::sources::oauth::TokenSet {
        access_token: "AT-0".into(),
        refresh_token: "RT-0".into(),
        expires_in: -100,
        token_type: "Bearer".into(),
        user_ref: String::new(),
    };
    store::save_auth(&pool, vault.as_ref(), ACME.id, &tokens)
        .await
        .unwrap();
    (pool, vault)
}

type Delivery = (String, String, Option<Verifier>);

#[derive(Default)]
struct Recorder {
    deliveries: Mutex<Vec<Delivery>>,
}

impl CallbackSink for Recorder {
    fn authorized(&self, source: &str, code: String, verifier: Option<Verifier>) {
        self.deliveries
            .lock()
            .unwrap()
            .push((source.to_string(), code, verifier));
    }

    fn timed_out(&self, _source: &str) {}
}

fn browser_get(port: u16, path: &str) -> u16 {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connexion");
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("delai");
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"
    )
    .expect("requete");
    let mut raw = String::new();
    stream.read_to_string(&mut raw).expect("reponse");
    raw.split_whitespace()
        .nth(1)
        .and_then(|c| c.parse().ok())
        .expect("code HTTP")
}

#[tokio::test]
async fn authorization_callback_and_exchange_carry_the_pkce_verifier_end_to_end() {
    let (pool, vault) = setup().await;
    let fake = FakeServer::start(|_: &Call| (200, FRESH.to_string()));

    let server = tiny_http::Server::http("127.0.0.1:0").expect("port local");
    let port = server.server_addr().to_ip().expect("ip").port();
    let flow = Arc::new(OAuthFlow::new());
    let sink = Arc::new(Recorder::default());
    let stop = Arc::new(AtomicBool::new(false));
    let handle = {
        let (flow, sink, stop) = (flow.clone(), sink.clone(), stop.clone());
        std::thread::spawn(move || {
            serve(
                &server,
                &flow,
                sink.as_ref(),
                Duration::from_millis(20),
                &stop,
            )
        })
    };

    let verifier = Verifier::generate().unwrap();
    let challenge = verifier.challenge();
    let expected_verifier = verifier.expose().to_string();
    let url = ACME.authorization_url("cid", "state-1", Some(&challenge));
    assert!(
        url.contains(&format!("code_challenge={challenge}")),
        "{url}"
    );
    assert!(url.contains("code_challenge_method=S256"), "{url}");
    assert!(
        url.contains("redirect_uri=http%3A%2F%2Flocalhost%3A17483%2Fcallback%2Facme"),
        "{url}"
    );
    flow.set_pending(ACME.id, "state-1".into(), Some(verifier));

    assert_eq!(
        browser_get(port, "/callback/acme?code=CODE-9&state=state-1"),
        200
    );
    stop.store(true, Ordering::SeqCst);
    handle.join().unwrap().unwrap();

    let (source, code, verifier) = sink.deliveries.lock().unwrap().remove(0);
    assert_eq!((source.as_str(), code.as_str()), ("acme", "CODE-9"));
    let verifier = verifier.expect("le verifier suit le code");
    assert_eq!(verifier.expose(), expected_verifier);

    exchange_and_store(
        &ACME,
        &fake.base_url,
        &pool,
        vault.as_ref(),
        &code,
        Some(&verifier),
    )
    .await
    .unwrap();

    let calls = fake.calls_to("/token");
    assert_eq!(calls.len(), 1);
    let call = &calls[0];
    assert_eq!(
        call.form_value("grant_type").as_deref(),
        Some("authorization_code")
    );
    assert_eq!(call.form_value("code").as_deref(), Some("CODE-9"));
    assert_eq!(
        call.form_value("code_verifier").as_deref(),
        Some(expected_verifier.as_str())
    );
    assert_eq!(
        call.form_value("redirect_uri").as_deref(),
        Some("http://localhost:17483/callback/acme")
    );
    assert_eq!(call.form_value("client_id").as_deref(), Some("cid"));
    let stored = store::load_auth(&pool, vault.as_ref(), ACME.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored.access_token.as_str(), "AT-1");
}

#[tokio::test]
async fn an_accepted_refresh_stores_the_new_tokens() {
    let (pool, vault) = setup().await;
    let fake = FakeServer::start(|_: &Call| (200, FRESH.to_string()));

    let access = refresh_locked(&ACME, &fake.base_url, &pool, vault.as_ref(), "AT-0")
        .await
        .unwrap();

    assert_eq!(access.as_str(), "AT-1");
    let call = &fake.calls_to("/token")[0];
    assert_eq!(
        call.form_value("grant_type").as_deref(),
        Some("refresh_token")
    );
    assert_eq!(call.form_value("refresh_token").as_deref(), Some("RT-0"));
    let stored = store::load_auth(&pool, vault.as_ref(), ACME.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored.refresh_token.as_str(), "RT-1");
}

#[tokio::test]
async fn a_refresh_reply_without_a_new_refresh_token_keeps_the_old_one() {
    let (pool, vault) = setup().await;
    let fake = FakeServer::start(|_: &Call| {
        (
            200,
            r#"{"access_token":"AT-2","expires_in":3600}"#.to_string(),
        )
    });

    refresh_locked(&ACME, &fake.base_url, &pool, vault.as_ref(), "AT-0")
        .await
        .unwrap();

    let stored = store::load_auth(&pool, vault.as_ref(), ACME.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored.access_token.as_str(), "AT-2");
    assert_eq!(stored.refresh_token.as_str(), "RT-0");
}

#[tokio::test]
async fn a_refused_refresh_reports_the_error_and_keeps_the_stored_tokens() {
    let (pool, vault) = setup().await;
    let fake = FakeServer::start(|_: &Call| (400, r#"{"error":"invalid_grant"}"#.to_string()));

    let err = refresh_locked(&ACME, &fake.base_url, &pool, vault.as_ref(), "AT-0")
        .await
        .unwrap_err();

    assert_eq!(err.kind(), "Unauthorized");
    assert!(err.to_string().contains("invalid_grant"), "{err}");
    let stored = store::load_auth(&pool, vault.as_ref(), ACME.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored.access_token.as_str(), "AT-0");
}

#[tokio::test]
async fn two_concurrent_refreshes_make_a_single_exchange() {
    let (pool, vault) = setup().await;
    let fake = FakeServer::start(|_: &Call| {
        std::thread::sleep(Duration::from_millis(80));
        (200, FRESH.to_string())
    });
    let base = fake.base_url.clone();

    let first = {
        let (pool, vault, base) = (pool.clone(), vault.clone(), base.clone());
        tokio::spawn(async move {
            refresh_locked(&ACME, &base, &pool, vault.as_ref(), "AT-0")
                .await
                .map(|t| t.to_string())
        })
    };
    let second = {
        let (pool, vault, base) = (pool.clone(), vault.clone(), base.clone());
        tokio::spawn(async move {
            refresh_locked(&ACME, &base, &pool, vault.as_ref(), "AT-0")
                .await
                .map(|t| t.to_string())
        })
    };

    assert_eq!(first.await.unwrap().unwrap(), "AT-1");
    assert_eq!(second.await.unwrap().unwrap(), "AT-1");
    assert_eq!(fake.calls_to("/token").len(), 1, "un seul echange");
}

#[tokio::test]
async fn a_rate_limited_token_request_is_retried_then_succeeds() {
    let hits = Arc::new(AtomicUsize::new(0));
    let seen = hits.clone();
    let fake = FakeServer::start(move |_: &Call| {
        if seen.fetch_add(1, Ordering::SeqCst) < 2 {
            (429, String::new())
        } else {
            (200, FRESH.to_string())
        }
    });

    let set = request_token_with(
        &ACME,
        &fake.base_url,
        &[("grant_type", "refresh_token")],
        "x",
        Duration::from_millis(1),
    )
    .await
    .unwrap();

    assert_eq!(set.access_token, "AT-1");
    assert_eq!(fake.calls_to("/token").len(), 3);
}

#[tokio::test]
async fn a_token_request_that_stays_rate_limited_gives_up_after_three_attempts() {
    let fake = FakeServer::start(|_: &Call| (429, String::new()));

    let err = request_token_with(
        &ACME,
        &fake.base_url,
        &[("grant_type", "refresh_token")],
        "x",
        Duration::from_millis(1),
    )
    .await
    .unwrap_err();

    assert_eq!(err.kind(), "RateLimit");
    assert_eq!(fake.calls_to("/token").len(), 3);
}

#[tokio::test]
async fn the_withings_spec_runs_through_the_same_generic_exchange() {
    let fake = FakeServer::start(|_: &Call| {
        (
            200,
            r#"{"status":0,"body":{"userid":"7","access_token":"AT-W","refresh_token":"RT-W","expires_in":10800,"token_type":"Bearer","scope":"user.info"}}"#
                .to_string(),
        )
    });
    let spec = &synk_lib::sources::withings::spec::OAUTH;

    let set = refresh_token(spec, &fake.base_url, "cid", "csecret", "RT-OLD")
        .await
        .unwrap();

    assert_eq!(set.access_token, "AT-W");
    let call = &fake.calls_to("/v2/oauth2")[0];
    assert_eq!(call.form_value("action").as_deref(), Some("requesttoken"));
    assert_eq!(
        call.form_value("grant_type").as_deref(),
        Some("refresh_token")
    );
}

static CAPTURED: Mutex<Vec<String>> = Mutex::new(Vec::new());
static INSTALL: Once = Once::new();

struct CaptureLogger;

impl log::Log for CaptureLogger {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        CAPTURED
            .lock()
            .unwrap()
            .push(format!("{} {}", record.level(), record.args()));
    }

    fn flush(&self) {}
}

fn install_capture() {
    INSTALL.call_once(|| {
        log::set_logger(&CaptureLogger).expect("aucun autre journal installe dans ce binaire");
        log::set_max_level(log::LevelFilter::Trace);
    });
}

fn captured_since(start: usize) -> Vec<String> {
    CAPTURED.lock().unwrap()[start..].to_vec()
}

#[tokio::test]
async fn no_secret_reaches_an_error_or_the_log_output_on_the_failure_paths() {
    install_capture();
    let start = CAPTURED.lock().unwrap().len();
    log::debug!("CONTROLE-INSTRUMENT");
    log::debug!("controle de fuite FUITE-VOLONTAIRE");
    let control = captured_since(start);
    assert!(
        control.iter().any(|l| l.contains("CONTROLE-INSTRUMENT")),
        "l'instrument ne voit pas le journal"
    );
    assert!(
        control.iter().any(|l| l.contains("FUITE-VOLONTAIRE")),
        "l'instrument ne voit pas une fuite plantee"
    );
    let start = CAPTURED.lock().unwrap().len();

    let verifier = Verifier::from_secret("VER-LEAKCHECK-0123456789");
    let unreadable = FakeServer::start(|_: &Call| {
        (
            200,
            "access_token=AT-LEAKCHECK&refresh_token=RT-LEAKCHECK&code_verifier=VER-LEAKCHECK-0123456789"
                .to_string(),
        )
    });
    let refused = FakeServer::start(|_: &Call| {
        (
            400,
            r#"{"error":"invalid_grant","refresh_token":"RT-LEAKCHECK","code_verifier":"VER-LEAKCHECK-0123456789"}"#
                .to_string(),
        )
    });
    let server_error =
        FakeServer::start(|_: &Call| (500, "boom access_token=AT-LEAKCHECK".to_string()));
    let withings_garbled = FakeServer::start(|_: &Call| {
        (
            200,
            "access_token=AT-LEAKCHECK&refresh_token=RT-LEAKCHECK".to_string(),
        )
    });

    let mut shown = Vec::new();
    for fake in [&unreadable, &refused, &server_error] {
        let err = exchange_code(
            &ACME,
            &fake.base_url,
            "cid",
            "csecret",
            "CODE-LEAKCHECK",
            Some(&verifier),
        )
        .await
        .unwrap_err();
        shown.push(err.to_string());
        shown.push(format!("{err:?}"));
        shown.push(err.redacted().to_string());
    }
    let err = exchange_code(
        &synk_lib::sources::withings::spec::OAUTH,
        &withings_garbled.base_url,
        "cid",
        "csecret",
        "CODE-LEAKCHECK",
        None,
    )
    .await
    .unwrap_err();
    shown.push(err.to_string());
    shown.push(format!("{verifier:?}"));

    for text in &shown {
        assert!(
            !text.contains("LEAKCHECK"),
            "fuite dans une erreur : {text}"
        );
    }
    let logs = captured_since(start);
    assert!(
        logs.iter().any(|l| l.contains("illisible")),
        "la voie de journal n'a pas ete exercee : {logs:?}"
    );
    for line in &logs {
        assert!(
            !line.contains("LEAKCHECK"),
            "fuite dans le journal : {line}"
        );
    }
}

#[allow(dead_code)]
fn wait_until(what: &str, condition: impl Fn() -> bool) {
    let started = Instant::now();
    while !condition() {
        assert!(started.elapsed() < Duration::from_secs(5), "delai : {what}");
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[tokio::test]
async fn a_basic_spec_sends_the_client_in_the_header_and_never_in_the_form() {
    use base64::Engine;
    let fake = FakeServer::start(|_: &Call| (200, FRESH.to_string()));

    exchange_code(
        &BASIC,
        &fake.base_url,
        "client-test",
        "secret-test",
        "code-test",
        None,
    )
    .await
    .unwrap();

    let call = &fake.calls_to("/token")[0];
    let expected = format!(
        "Basic {}",
        base64::engine::general_purpose::STANDARD.encode("client-test:secret-test")
    );
    assert_eq!(call.authorization.as_deref(), Some(expected.as_str()));
    assert_eq!(call.form_value("client_id"), None);
    assert_eq!(call.form_value("client_secret"), None);
    assert_eq!(call.form_value("code").as_deref(), Some("code-test"));
    assert_eq!(
        call.form_value("grant_type").as_deref(),
        Some("authorization_code")
    );
}

#[tokio::test]
async fn a_body_spec_still_sends_the_client_in_the_form() {
    let fake = FakeServer::start(|_: &Call| (200, FRESH.to_string()));

    exchange_code(
        &ACME,
        &fake.base_url,
        "client-test",
        "secret-test",
        "code-test",
        None,
    )
    .await
    .unwrap();

    let call = &fake.calls_to("/token")[0];
    assert_eq!(call.authorization, None);
    assert_eq!(call.form_value("client_id").as_deref(), Some("client-test"));
    assert_eq!(
        call.form_value("client_secret").as_deref(),
        Some("secret-test")
    );
}
