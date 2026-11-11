use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use super::pkce::Verifier;
use super::spec::AUTH_TIMEOUT;
use crate::error::AppError;
use crate::sources::SourceId;

struct Pending {
    state: String,
    verifier: Option<Verifier>,
    deadline: Instant,
}

pub struct OAuthFlow {
    pending: Mutex<HashMap<String, Pending>>,
    pub server_started: AtomicBool,
}

pub struct Attempt {
    pub state: String,
    pub verifier: Option<Verifier>,
}

impl OAuthFlow {
    pub fn new() -> OAuthFlow {
        OAuthFlow {
            pending: Mutex::new(HashMap::new()),
            server_started: AtomicBool::new(false),
        }
    }

    pub fn global() -> &'static OAuthFlow {
        static FLOW: OnceLock<OAuthFlow> = OnceLock::new();
        FLOW.get_or_init(OAuthFlow::new)
    }

    pub fn set_pending(&self, source: SourceId, state: String, verifier: Option<Verifier>) {
        self.set_pending_for(source, state, verifier, AUTH_TIMEOUT);
    }

    pub fn set_pending_for(
        &self,
        source: SourceId,
        state: String,
        verifier: Option<Verifier>,
        timeout: Duration,
    ) {
        lock(&self.pending).insert(
            source.as_str().to_string(),
            Pending {
                state,
                verifier,
                deadline: Instant::now() + timeout,
            },
        );
    }

    pub fn has_pending(&self) -> bool {
        !lock(&self.pending).is_empty()
    }

    pub fn clear(&self, source: &str) {
        lock(&self.pending).remove(source);
    }

    pub fn current_state(&self, source: &str) -> Option<String> {
        lock(&self.pending).get(source).map(|p| p.state.clone())
    }

    pub fn take(&self, source: &str) -> Option<Attempt> {
        lock(&self.pending).remove(source).map(|p| Attempt {
            state: p.state,
            verifier: p.verifier,
        })
    }

    pub fn take_expired(&self) -> Vec<String> {
        let now = Instant::now();
        let mut pending = lock(&self.pending);
        let expired: Vec<String> = pending
            .iter()
            .filter(|(_, p)| now > p.deadline)
            .map(|(id, _)| id.clone())
            .collect();
        for id in &expired {
            pending.remove(id);
        }
        expired
    }
}

impl Default for OAuthFlow {
    fn default() -> Self {
        OAuthFlow::new()
    }
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub fn generate_state() -> Result<String, AppError> {
    let mut bytes = [0u8; 24];
    getrandom::fill(&mut bytes).map_err(|e| AppError::Crypto(format!("state rng: {e}")))?;
    Ok(bytes.iter().map(|b| format!("{:02x}", b)).collect())
}

#[derive(Debug, PartialEq, Eq)]
pub enum CallbackOutcome {
    NotCallback,
    InvalidState,
    MissingCode,
    Denied(String),
    Success(String),
}

fn bare_path_owner(path: &str) -> Option<&'static str> {
    crate::sources::all_sources()
        .iter()
        .find_map(|source| match source.kind() {
            crate::sources::SourceKind::OAuth(spec) if spec.redirect_path == Some(path) => {
                Some(spec.id.as_str())
            }
            _ => None,
        })
}

pub fn route_callback(url: &str) -> Option<(&str, &str)> {
    let (path, query) = url.split_once('?').unwrap_or((url, ""));
    if let Some(owner) = bare_path_owner(path) {
        return Some((owner, query));
    }
    let source = path.strip_prefix("/callback/")?;
    if source.is_empty() || source.contains('/') {
        return None;
    }
    Some((source, query))
}

const MAX_REASON: usize = 64;

fn decoded(raw: &str) -> String {
    urlencoding::decode(raw).map_or_else(|_| raw.to_string(), |text| text.into_owned())
}

fn clean_reason(raw: &str) -> String {
    decoded(raw)
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
        .take(MAX_REASON)
        .collect()
}

pub fn evaluate_query(query: &str, expected: Option<&str>) -> CallbackOutcome {
    let mut code_param = None;
    let mut state_param = None;
    let mut error_param = None;
    for pair in query.split('&') {
        let mut kv = pair.splitn(2, '=');
        let key = kv.next().unwrap_or("");
        let val = kv.next().unwrap_or("");
        match key {
            "code" => code_param = Some(decoded(val)),
            "state" => state_param = Some(decoded(val)),
            "error" => error_param = Some(clean_reason(val)),
            _ => {}
        }
    }

    match expected {
        Some(exp) if state_param.as_deref() == Some(exp) => {}
        _ => return CallbackOutcome::InvalidState,
    }

    if let Some(reason) = error_param.filter(|r| !r.is_empty()) {
        return CallbackOutcome::Denied(reason);
    }

    match code_param {
        Some(code) if !code.is_empty() => CallbackOutcome::Success(code),
        _ => CallbackOutcome::MissingCode,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WITHINGS: SourceId = SourceId::WITHINGS;
    const OURA: SourceId = SourceId::new("oura");

    fn evaluate(url: &str, flow: &OAuthFlow) -> (Option<String>, CallbackOutcome) {
        match route_callback(url) {
            None => (None, CallbackOutcome::NotCallback),
            Some((source, query)) => (
                Some(source.to_string()),
                evaluate_query(query, flow.current_state(source).as_deref()),
            ),
        }
    }

    #[test]
    fn the_legacy_path_is_routed_to_withings() {
        assert_eq!(
            route_callback("/callback?code=a&state=b"),
            Some(("withings", "code=a&state=b"))
        );
    }

    #[test]
    fn a_path_per_source_is_routed_to_that_source() {
        assert_eq!(
            route_callback("/callback/oura?code=a&state=b"),
            Some(("oura", "code=a&state=b"))
        );
        assert_eq!(route_callback("/callback/polar"), Some(("polar", "")));
    }

    #[test]
    fn other_paths_are_not_callbacks() {
        for url in [
            "/favicon.ico",
            "/callbackx?code=a",
            "/callback/",
            "/callback/a/b",
            "/",
            "/oauth/callback",
        ] {
            assert_eq!(route_callback(url), None, "{url}");
        }
    }

    #[test]
    fn valid_state_returns_code() {
        assert_eq!(
            evaluate_query("code=abc123&state=deadbeef", Some("deadbeef")),
            CallbackOutcome::Success("abc123".to_string())
        );
    }

    #[test]
    fn state_param_order_independent() {
        assert_eq!(
            evaluate_query("state=tok&code=xyz", Some("tok")),
            CallbackOutcome::Success("xyz".to_string())
        );
    }

    #[test]
    fn stale_state_is_rejected() {
        assert_eq!(
            evaluate_query("code=abc&state=OLD", Some("NEW")),
            CallbackOutcome::InvalidState
        );
    }

    #[test]
    fn latest_state_after_retry_is_accepted() {
        assert_eq!(
            evaluate_query("code=abc&state=NEW", Some("NEW")),
            CallbackOutcome::Success("abc".to_string())
        );
    }

    #[test]
    fn no_pending_state_is_rejected() {
        assert_eq!(
            evaluate_query("code=abc&state=x", None),
            CallbackOutcome::InvalidState
        );
    }

    #[test]
    fn missing_code_detected() {
        assert_eq!(
            evaluate_query("state=tok", Some("tok")),
            CallbackOutcome::MissingCode
        );
    }

    #[test]
    fn empty_code_detected() {
        assert_eq!(
            evaluate_query("code=&state=tok", Some("tok")),
            CallbackOutcome::MissingCode
        );
    }

    #[test]
    fn generate_state_is_48_hex_chars() {
        let s = generate_state().expect("entropie");
        assert_eq!(s.len(), 48);
        assert!(s.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn two_generated_states_differ() {
        assert_ne!(
            generate_state().expect("entropie"),
            generate_state().expect("entropie")
        );
    }

    #[test]
    fn flow_set_pending_keeps_latest_state() {
        let flow = OAuthFlow::new();
        flow.set_pending(WITHINGS, "first".to_string(), None);
        flow.set_pending(WITHINGS, "second".to_string(), None);
        assert_eq!(flow.current_state("withings").as_deref(), Some("second"));
        flow.clear("withings");
        assert_eq!(flow.current_state("withings"), None);
    }

    #[test]
    fn each_source_keeps_its_own_state_and_verifier() {
        let flow = OAuthFlow::new();
        flow.set_pending(
            WITHINGS,
            "w-state".to_string(),
            Some(Verifier::from_secret("w-ver")),
        );
        flow.set_pending(
            OURA,
            "o-state".to_string(),
            Some(Verifier::from_secret("o-ver")),
        );
        assert_eq!(
            evaluate("/callback?code=c&state=o-state", &flow).1,
            CallbackOutcome::InvalidState,
            "le state d'une source ne passe pas sur le chemin d'une autre"
        );
        assert_eq!(
            evaluate("/callback/oura?code=c&state=o-state", &flow).1,
            CallbackOutcome::Success("c".to_string())
        );
        let attempt = flow.take("oura").expect("tentative");
        assert_eq!(attempt.verifier.expect("verifier").expose(), "o-ver");
        assert_eq!(flow.current_state("oura"), None);
        assert_eq!(flow.current_state("withings").as_deref(), Some("w-state"));
    }

    #[test]
    fn only_expired_attempts_are_taken_and_each_once() {
        let flow = OAuthFlow::new();
        flow.set_pending_for(WITHINGS, "a".into(), None, Duration::from_millis(0));
        flow.set_pending_for(OURA, "b".into(), None, Duration::from_secs(60));
        std::thread::sleep(Duration::from_millis(5));
        assert_eq!(flow.take_expired(), vec!["withings".to_string()]);
        assert!(flow.take_expired().is_empty());
        assert_eq!(flow.current_state("oura").as_deref(), Some("b"));
    }

    #[test]
    fn a_refused_consent_with_the_right_state_is_denied_with_its_reason() {
        assert_eq!(
            evaluate_query(
                "error=access_denied&error_description=The+user+said+no&state=tok",
                Some("tok")
            ),
            CallbackOutcome::Denied("access_denied".to_string())
        );
        assert_eq!(
            evaluate_query("error=access_denied&state=OLD", Some("tok")),
            CallbackOutcome::InvalidState
        );
    }

    #[test]
    fn the_denial_reason_is_bounded_and_cleaned() {
        let long = "x".repeat(500);
        let CallbackOutcome::Denied(reason) =
            evaluate_query(&format!("error={long}&state=tok"), Some("tok"))
        else {
            panic!("refus attendu");
        };
        assert!(reason.chars().count() <= 64, "{}", reason.len());
        let CallbackOutcome::Denied(reason) =
            evaluate_query("error=%3Cscript%3E&state=tok", Some("tok"))
        else {
            panic!("refus attendu");
        };
        assert!(!reason.contains('<'), "{reason}");
    }

    #[test]
    fn a_percent_encoded_code_is_decoded_once() {
        assert_eq!(
            evaluate_query("code=4%2F0AbC%3Dd&state=tok", Some("tok")),
            CallbackOutcome::Success("4/0AbC=d".to_string())
        );
        assert_eq!(
            evaluate_query("code=4/0AbC&state=tok", Some("tok")),
            CallbackOutcome::Success("4/0AbC".to_string())
        );
    }
}
