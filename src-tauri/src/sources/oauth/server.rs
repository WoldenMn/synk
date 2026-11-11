use sqlx::SqlitePool;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

use super::connect::exchange_and_store;
use super::flow::{evaluate_query, route_callback, CallbackOutcome, OAuthFlow};
use super::pkce::Verifier;
use super::spec::{AUTH_TIMEOUT, CALLBACK_PORT};
use crate::crypto::CryptoVault;
use crate::error::AppError;
use crate::sources::{find_source, SourceId, SourceKind};

const POLL_INTERVAL: Duration = Duration::from_secs(2);
static NEVER_STOP: AtomicBool = AtomicBool::new(false);

pub trait CallbackSink {
    fn authorized(&self, source: &str, code: String, verifier: Option<Verifier>);
    fn timed_out(&self, source: &str);
    fn denied(&self, _source: &str, _reason: &str) {}
}

struct AppSink {
    pool: SqlitePool,
    vault: Arc<CryptoVault>,
    app: AppHandle,
}

impl CallbackSink for AppSink {
    fn authorized(&self, source: &str, code: String, verifier: Option<Verifier>) {
        tauri::async_runtime::spawn(complete_exchange(
            self.pool.clone(),
            self.vault.clone(),
            self.app.clone(),
            source.to_string(),
            code,
            verifier,
        ));
    }

    fn timed_out(&self, source: &str) {
        emit_or_warn(&self.app, &event_name(source, "auth-timeout"), ());
    }

    fn denied(&self, source: &str, reason: &str) {
        let error = AppError::Forbidden(format!(
            "{} : autorisation refusee ({reason})",
            crate::sources::find_source(source).map_or(source, |s| s.label())
        ));
        emit_or_warn(
            &self.app,
            &event_name(source, "auth-error"),
            error.into_event_payload(),
        );
    }
}

fn event_name(source: &str, suffix: &str) -> String {
    format!("{source}-{suffix}")
}

pub fn bind_loopback(
    port: u16,
) -> Result<(tiny_http::Server, Option<tiny_http::Server>), std::io::Error> {
    let v4 = tiny_http::Server::http(("127.0.0.1", port))
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    let v6 = match tiny_http::Server::http(("::1", port)) {
        Ok(server) => Some(server),
        Err(e) => {
            let in_use = e
                .downcast_ref::<std::io::Error>()
                .is_some_and(|io| io.kind() == std::io::ErrorKind::AddrInUse);
            if in_use {
                return Err(std::io::Error::other(
                    "le port de rappel est deja pris sur ::1 par un autre programme",
                ));
            }
            log::info!("[oauth] ecoute sur ::1 indisponible : {e}");
            None
        }
    };
    Ok((v4, v6))
}

pub fn serve_both(
    listeners: (tiny_http::Server, Option<tiny_http::Server>),
    flow: &OAuthFlow,
    sink: &(dyn CallbackSink + Sync),
    poll: Duration,
) {
    let (v4, v6) = listeners;
    std::thread::scope(|scope| {
        if let Some(v6) = &v6 {
            scope.spawn(move || {
                if let Err(e) = serve_until_idle(v6, flow, sink, poll) {
                    log::error!("OAuth callback server ([::1]) stopped on recv error: {e}");
                }
            });
        }
        if let Err(e) = serve_until_idle(&v4, flow, sink, poll) {
            log::error!("OAuth callback server stopped on recv error: {e}");
        }
    });
}

pub fn spawn_callback_server(
    starter: SourceId,
    pool: SqlitePool,
    vault: Arc<CryptoVault>,
    app: AppHandle,
) {
    std::thread::spawn(move || {
        let flow = OAuthFlow::global();
        let sink = AppSink { pool, vault, app };
        loop {
            match bind_loopback(CALLBACK_PORT) {
                Ok(listeners) => serve_both(listeners, flow, &sink, POLL_INTERVAL),
                Err(e) => {
                    log::error!("Failed to start OAuth callback server: {}", e);
                    emit_or_warn(
                        &sink.app,
                        &event_name(starter.as_str(), "auth-error"),
                        AppError::Internal(format!("callback server: {}", e)).into_event_payload(),
                    );
                    flow.clear(starter.as_str());
                    flow.server_started.store(false, Ordering::SeqCst);
                    return;
                }
            }
            flow.server_started.store(false, Ordering::SeqCst);
            let restart = flow.has_pending()
                && flow
                    .server_started
                    .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                    .is_ok();
            if !restart {
                return;
            }
        }
    });
}

pub fn serve(
    server: &tiny_http::Server,
    flow: &OAuthFlow,
    sink: &dyn CallbackSink,
    poll: Duration,
    stop: &AtomicBool,
) -> std::io::Result<()> {
    run(server, flow, sink, poll, stop, false)
}

pub fn serve_until_idle(
    server: &tiny_http::Server,
    flow: &OAuthFlow,
    sink: &dyn CallbackSink,
    poll: Duration,
) -> std::io::Result<()> {
    run(server, flow, sink, poll, &NEVER_STOP, true)
}

fn run(
    server: &tiny_http::Server,
    flow: &OAuthFlow,
    sink: &dyn CallbackSink,
    poll: Duration,
    stop: &AtomicBool,
    until_idle: bool,
) -> std::io::Result<()> {
    while !stop.load(Ordering::SeqCst) {
        expire_pending(flow, sink);
        if until_idle && !flow.has_pending() {
            break;
        }

        let request = match server.recv_timeout(poll)? {
            Some(r) => r,
            None => continue,
        };

        let url = request.url().to_string();
        let Some((source, query)) = route_callback(&url) else {
            respond_to(request, &CallbackOutcome::NotCallback);
            continue;
        };
        let outcome = evaluate_query(query, flow.current_state(source).as_deref());
        if let CallbackOutcome::Success(code) = &outcome {
            let verifier = flow.take(source).and_then(|attempt| attempt.verifier);
            sink.authorized(source, code.clone(), verifier);
        }
        if let CallbackOutcome::Denied(reason) = &outcome {
            flow.take(source);
            sink.denied(source, reason);
        }
        respond_to(request, &outcome);
    }
    Ok(())
}

fn expire_pending(flow: &OAuthFlow, sink: &dyn CallbackSink) {
    for source in flow.take_expired() {
        log::warn!(
            "OAuth callback for {source} timed out after {}s",
            AUTH_TIMEOUT.as_secs()
        );
        sink.timed_out(&source);
    }
}

fn respond_to(request: tiny_http::Request, outcome: &CallbackOutcome) {
    let response = match outcome {
        CallbackOutcome::NotCallback => {
            tiny_http::Response::from_string("OK").with_status_code(tiny_http::StatusCode(200))
        }
        CallbackOutcome::InvalidState => html_response("Erreur : state invalide", 400),
        CallbackOutcome::MissingCode => html_response("Erreur : code manquant", 400),
        CallbackOutcome::Denied(_) => html_response("Autorisation refusée", 400),
        CallbackOutcome::Success(_) => success_response(),
    };
    respond_or_warn(request, response);
}

async fn complete_exchange(
    pool: SqlitePool,
    vault: Arc<CryptoVault>,
    app: AppHandle,
    source: String,
    code: String,
    verifier: Option<Verifier>,
) {
    let outcome = match find_source(&source).map(|s| s.kind()) {
        Some(SourceKind::OAuth(spec)) => {
            exchange_and_store(
                spec,
                spec.api_base,
                &pool,
                vault.as_ref(),
                &code,
                verifier.as_ref(),
            )
            .await
        }
        _ => Err(AppError::Internal(format!(
            "source {source} sans autorisation OAuth"
        ))),
    };
    match outcome {
        Ok(()) => emit_or_warn(&app, &event_name(&source, "auth-success"), ()),
        Err(e) => {
            let e = e.redacted();
            log::error!("Authorization for {source} failed: {}", e);
            emit_or_warn(
                &app,
                &event_name(&source, "auth-error"),
                e.into_event_payload(),
            );
        }
    }
}

fn html_header() -> tiny_http::Header {
    "Content-Type: text/html; charset=utf-8"
        .parse::<tiny_http::Header>()
        .expect("static content-type header is valid")
}

fn emit_or_warn<S: serde::Serialize + Clone>(app: &AppHandle, event: &str, payload: S) {
    if let Err(e) = app.emit(event, payload) {
        log::warn!("[oauth] evenement {event} non emis : {e}");
    }
}

fn respond_or_warn<R: std::io::Read>(
    request: tiny_http::Request,
    response: tiny_http::Response<R>,
) {
    if let Err(e) = request.respond(response) {
        log::warn!("[oauth] reponse au navigateur non envoyee : {e}");
    }
}

fn html_response(title: &str, status: u16) -> tiny_http::Response<std::io::Cursor<Vec<u8>>> {
    tiny_http::Response::from_string(format!("<html><body><h1>{}</h1></body></html>", title))
        .with_header(html_header())
        .with_status_code(tiny_http::StatusCode(status))
}

fn success_response() -> tiny_http::Response<std::io::Cursor<Vec<u8>>> {
    tiny_http::Response::from_string(
        "<html><body style='font-family:sans-serif;text-align:center;padding:60px'>\
         <h1 style='color:#22c55e'>Authentification reussie !</h1>\
         <p>Vous pouvez fermer cette fenetre et retourner dans Synk.</p></body></html>",
    )
    .with_header(html_header())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::sync::atomic::AtomicUsize;
    use std::sync::Mutex;
    use std::time::Instant;

    const TEST_POLL: Duration = Duration::from_millis(20);
    const WITHINGS: SourceId = SourceId::WITHINGS;
    const OURA: SourceId = SourceId::new("oura");

    type Delivery = (String, String, Option<String>);

    #[derive(Default)]
    struct Recorder {
        deliveries: Mutex<Vec<Delivery>>,
        timed_out: Mutex<Vec<String>>,
        timeout_count: AtomicUsize,
        refused: Mutex<Vec<(String, String)>>,
    }

    impl Recorder {
        fn codes(&self) -> Vec<String> {
            self.deliveries
                .lock()
                .unwrap()
                .iter()
                .map(|(_, code, _)| code.clone())
                .collect()
        }

        fn deliveries(&self) -> Vec<Delivery> {
            self.deliveries.lock().unwrap().clone()
        }

        fn timeouts(&self) -> usize {
            self.timeout_count.load(Ordering::SeqCst)
        }

        fn timed_out_sources(&self) -> Vec<String> {
            self.timed_out.lock().unwrap().clone()
        }
    }

    impl CallbackSink for Recorder {
        fn authorized(&self, source: &str, code: String, verifier: Option<Verifier>) {
            self.deliveries.lock().unwrap().push((
                source.to_string(),
                code,
                verifier.map(|v| v.expose().to_string()),
            ));
        }

        fn denied(&self, source: &str, reason: &str) {
            self.refused
                .lock()
                .unwrap()
                .push((source.to_string(), reason.to_string()));
        }

        fn timed_out(&self, source: &str) {
            self.timed_out.lock().unwrap().push(source.to_string());
            self.timeout_count.fetch_add(1, Ordering::SeqCst);
        }
    }

    struct Harness {
        port: u16,
        flow: Arc<OAuthFlow>,
        sink: Arc<Recorder>,
        stop: Arc<AtomicBool>,
        handle: Option<std::thread::JoinHandle<std::io::Result<()>>>,
    }

    impl Harness {
        fn start() -> Harness {
            let server = tiny_http::Server::http("127.0.0.1:0").expect("port local ephemere");
            let port = server.server_addr().to_ip().expect("adresse IP").port();
            let flow = Arc::new(OAuthFlow::new());
            let sink = Arc::new(Recorder::default());
            let stop = Arc::new(AtomicBool::new(false));
            let handle = {
                let (flow, sink, stop) = (flow.clone(), sink.clone(), stop.clone());
                std::thread::spawn(move || serve(&server, &flow, sink.as_ref(), TEST_POLL, &stop))
            };
            Harness {
                port,
                flow,
                sink,
                stop,
                handle: Some(handle),
            }
        }

        fn get(&self, path: &str) -> (u16, String) {
            let mut stream = TcpStream::connect(("127.0.0.1", self.port)).expect("connexion");
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .expect("delai de lecture");
            write!(
                stream,
                "GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"
            )
            .expect("requete");
            let mut raw = String::new();
            stream.read_to_string(&mut raw).expect("reponse");
            let status = raw
                .split_whitespace()
                .nth(1)
                .and_then(|code| code.parse().ok())
                .expect("code HTTP");
            let body = raw.split("\r\n\r\n").nth(1).unwrap_or("").to_string();
            (status, body)
        }

        fn wait_until(&self, what: &str, condition: impl Fn() -> bool) {
            let started = Instant::now();
            while !condition() {
                assert!(
                    started.elapsed() < Duration::from_secs(5),
                    "delai depasse : {what}"
                );
                std::thread::sleep(Duration::from_millis(10));
            }
        }

        fn finish(mut self) -> std::io::Result<()> {
            self.stop.store(true, Ordering::SeqCst);
            self.handle
                .take()
                .expect("fil du serveur")
                .join()
                .expect("le serveur ne doit pas paniquer")
        }
    }

    impl Drop for Harness {
        fn drop(&mut self) {
            self.stop.store(true, Ordering::SeqCst);
            if let Some(handle) = self.handle.take() {
                if handle.join().is_err() {
                    eprintln!("le fil du serveur de test a paniqué");
                }
            }
        }
    }

    #[test]
    fn a_valid_callback_hands_the_code_over_before_answering_and_ends_the_attempt() {
        let h = Harness::start();
        h.flow.set_pending(WITHINGS, "tok".to_string(), None);
        let (status, body) = h.get("/callback?code=abc123&state=tok");
        assert_eq!(status, 200);
        assert!(body.contains("Authentification reussie"), "{body}");
        assert_eq!(
            h.sink.codes(),
            vec!["abc123".to_string()],
            "le code est remis avant la reponse HTTP, sans attente"
        );
        assert_eq!(h.flow.current_state("withings"), None);
        let (status, _) = h.get("/callback?code=abc123&state=tok");
        assert_eq!(status, 400, "le state ne sert qu'une fois");
        assert_eq!(h.sink.codes().len(), 1);
    }

    #[test]
    fn the_legacy_path_reaches_withings_and_a_per_source_path_reaches_that_source() {
        let h = Harness::start();
        h.flow.set_pending(WITHINGS, "w".to_string(), None);
        h.flow.set_pending(
            OURA,
            "o".to_string(),
            Some(Verifier::from_secret("ver-oura")),
        );
        let (status, _) = h.get("/callback?code=cw&state=w");
        assert_eq!(status, 200);
        let (status, _) = h.get("/callback/oura?code=co&state=o");
        assert_eq!(status, 200);
        assert_eq!(
            h.sink.deliveries(),
            vec![
                ("withings".to_string(), "cw".to_string(), None),
                (
                    "oura".to_string(),
                    "co".to_string(),
                    Some("ver-oura".to_string())
                ),
            ]
        );
    }

    #[test]
    fn a_state_pending_for_one_source_is_refused_on_another_sources_path() {
        let h = Harness::start();
        h.flow.set_pending(WITHINGS, "w".to_string(), None);
        let (status, body) = h.get("/callback/oura?code=c&state=w");
        assert_eq!(status, 400);
        assert!(body.contains("state invalide"), "{body}");
        assert!(h.sink.codes().is_empty());
        assert_eq!(h.flow.current_state("withings").as_deref(), Some("w"));
    }

    #[test]
    fn a_favicon_request_is_ignored_and_keeps_the_attempt_alive() {
        let h = Harness::start();
        h.flow.set_pending(WITHINGS, "tok".to_string(), None);
        let (status, body) = h.get("/favicon.ico");
        assert_eq!((status, body.as_str()), (200, "OK"));
        assert!(h.sink.codes().is_empty());
        assert_eq!(h.flow.current_state("withings").as_deref(), Some("tok"));
        let (status, _) = h.get("/callback?code=abc&state=tok");
        assert_eq!(status, 200);
        assert_eq!(h.sink.codes(), vec!["abc".to_string()]);
    }

    #[test]
    fn a_wrong_state_is_refused_without_a_code_and_without_ending_the_attempt() {
        let h = Harness::start();
        h.flow.set_pending(WITHINGS, "good".to_string(), None);
        let (status, body) = h.get("/callback?code=abc&state=forged");
        assert_eq!(status, 400);
        assert!(body.contains("state invalide"), "{body}");
        assert!(h.sink.codes().is_empty());
        assert_eq!(h.flow.current_state("withings").as_deref(), Some("good"));
    }

    #[test]
    fn a_callback_without_any_pending_attempt_is_refused() {
        let h = Harness::start();
        let (status, body) = h.get("/callback?code=abc&state=anything");
        assert_eq!(status, 400);
        assert!(body.contains("state invalide"), "{body}");
        assert!(h.sink.codes().is_empty());
    }

    #[test]
    fn a_callback_without_a_code_is_refused() {
        let h = Harness::start();
        h.flow.set_pending(WITHINGS, "tok".to_string(), None);
        let (status, body) = h.get("/callback?state=tok");
        assert_eq!(status, 400);
        assert!(body.contains("code manquant"), "{body}");
        assert!(h.sink.codes().is_empty());
    }

    #[test]
    fn only_the_latest_attempt_state_is_accepted_by_the_same_server() {
        let h = Harness::start();
        h.flow.set_pending(WITHINGS, "first".to_string(), None);
        h.flow.set_pending(WITHINGS, "second".to_string(), None);
        let (status, _) = h.get("/callback?code=abc&state=first");
        assert_eq!(status, 400);
        let (status, _) = h.get("/callback?code=abc&state=second");
        assert_eq!(status, 200);
        assert_eq!(h.sink.codes(), vec!["abc".to_string()]);
    }

    #[test]
    fn an_expired_attempt_is_reported_once_with_its_source() {
        let h = Harness::start();
        h.flow
            .set_pending_for(OURA, "tok".to_string(), None, Duration::from_millis(60));
        h.wait_until("delai signale", || h.sink.timeouts() == 1);
        assert_eq!(h.sink.timed_out_sources(), vec!["oura".to_string()]);
        assert_eq!(h.flow.current_state("oura"), None);
        std::thread::sleep(TEST_POLL * 5);
        assert_eq!(h.sink.timeouts(), 1, "le delai ne se signale qu'une fois");
        let (status, _) = h.get("/callback/oura?code=abc&state=tok");
        assert_eq!(status, 400);
        assert!(h.sink.codes().is_empty());
    }

    #[test]
    fn a_completed_attempt_never_reports_a_timeout() {
        let h = Harness::start();
        h.flow.set_pending_for(
            WITHINGS,
            "tok".to_string(),
            None,
            Duration::from_millis(1000),
        );
        let (status, _) = h.get("/callback?code=abc&state=tok");
        assert_eq!(status, 200);
        std::thread::sleep(Duration::from_millis(1300));
        assert_eq!(h.sink.timeouts(), 0);
    }

    #[test]
    fn the_server_stops_cleanly_when_asked() {
        let h = Harness::start();
        let (status, _) = h.get("/favicon.ico");
        assert_eq!(status, 200);
        h.finish().expect("arret propre");
    }

    #[test]
    fn the_listener_stops_once_the_last_attempt_is_delivered_or_expired() {
        let flow = OAuthFlow::new();
        let sink = Recorder::default();
        let server = tiny_http::Server::http("127.0.0.1:0").expect("port local");
        let port = server.server_addr().to_ip().expect("adresse IP").port();
        flow.set_pending_for(
            WITHINGS,
            "etat-attendu".into(),
            None,
            Duration::from_secs(30),
        );
        let started = Instant::now();
        let client = std::thread::spawn(move || {
            let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connexion");
            write!(
                stream,
                "GET /callback?code=abc&state=etat-attendu HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"
            )
            .expect("requete");
            let mut raw = String::new();
            stream.read_to_string(&mut raw).expect("reponse");
            raw
        });
        serve_until_idle(&server, &flow, &sink, TEST_POLL).unwrap();
        assert!(client.join().unwrap().contains("200"));
        assert!(started.elapsed() < Duration::from_secs(5));
        assert_eq!(sink.codes(), vec!["abc".to_string()]);
        assert!(!flow.has_pending());

        flow.set_pending_for(OURA, "autre".into(), None, Duration::from_millis(50));
        serve_until_idle(&server, &flow, &sink, TEST_POLL).unwrap();
        assert_eq!(sink.timed_out_sources(), vec!["oura".to_string()]);
    }

    #[test]
    fn the_callback_port_is_held_on_ipv4_and_on_ipv6_so_that_localhost_cannot_be_hijacked() {
        let (v4, v6) = bind_loopback(0).expect("liaison");
        assert!(v4.server_addr().to_ip().unwrap().is_ipv4());
        let v6_available = std::net::TcpListener::bind("[::1]:0").is_ok();
        assert_eq!(v6.is_some(), v6_available);
        if let Some(v6) = v6 {
            assert!(v6.server_addr().to_ip().unwrap().is_ipv6());
        }
    }

    #[test]
    fn a_refused_consent_ends_the_attempt_at_once_and_tells_the_sink() {
        let h = Harness::start();
        h.flow.set_pending(OURA, "tok".into(), None);
        let (status, body) = h.get("/callback/oura?error=access_denied&state=tok");
        assert_eq!(status, 400);
        assert!(body.contains("refus"), "{body}");
        h.wait_until("le refus est remis", || {
            !h.sink.refused.lock().unwrap().is_empty()
        });
        assert_eq!(
            h.sink.refused.lock().unwrap().clone(),
            vec![("oura".to_string(), "access_denied".to_string())]
        );
        assert!(h.flow.current_state("oura").is_none());
        assert!(h.sink.codes().is_empty());
    }
}
