use std::time::Duration;

use super::pkce::Verifier;
use super::spec::{ClientAuth, OAuthSpec, TokenSet};
use crate::error::AppError;
use crate::utils::redact::redact;
use crate::utils::text::truncate_chars;

const TOKEN_ATTEMPTS: u32 = 3;
const TOKEN_BACKOFF: Duration = Duration::from_millis(1000);

#[derive(serde::Deserialize)]
struct StandardReply {
    access_token: String,
    #[serde(default)]
    refresh_token: String,
    #[serde(default = "default_expiry")]
    expires_in: i64,
    #[serde(default = "default_token_type")]
    token_type: String,
    #[serde(default)]
    user_id: Option<serde_json::Value>,
}

#[derive(serde::Deserialize)]
struct StandardError {
    error: Option<String>,
}

fn default_expiry() -> i64 {
    3600
}

fn default_token_type() -> String {
    "Bearer".to_string()
}

pub fn standard_json(status: u16, body: &str, refused: &str) -> Result<TokenSet, AppError> {
    match status {
        200..=299 => {}
        429 => return Err(AppError::RateLimit(format!("HTTP 429: {refused}"))),
        400 | 401 => {
            let code = serde_json::from_str::<StandardError>(body)
                .ok()
                .and_then(|e| e.error)
                .filter(|c| c.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '_'))
                .unwrap_or_else(|| "sans code".to_string());
            return Err(AppError::Unauthorized(format!(
                "HTTP {status}: {refused} ({code})"
            )));
        }
        other => {
            return Err(AppError::Internal(format!(
                "HTTP {other}: token request ({} octets)",
                body.len()
            )))
        }
    }
    let reply: StandardReply = serde_json::from_str(body).map_err(|_| unreadable(body))?;
    Ok(TokenSet {
        access_token: reply.access_token,
        refresh_token: reply.refresh_token,
        expires_in: reply.expires_in,
        token_type: reply.token_type,
        user_ref: match reply.user_id {
            Some(serde_json::Value::String(s)) => s,
            Some(other) => other.to_string(),
            None => String::new(),
        },
    })
}

pub async fn exchange_code(
    spec: &OAuthSpec,
    base: &str,
    client_id: &str,
    client_secret: &str,
    code: &str,
    verifier: Option<&Verifier>,
) -> Result<TokenSet, AppError> {
    let redirect = spec.redirect_uri();
    let mut form: Vec<(&str, &str)> = vec![
        ("grant_type", "authorization_code"),
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("code", code),
        ("redirect_uri", &redirect),
    ];
    if let Some(v) = verifier {
        form.push(("code_verifier", v.expose()));
    }
    request_token(spec, base, &form, "token exchange refused").await
}

pub async fn refresh_token(
    spec: &OAuthSpec,
    base: &str,
    client_id: &str,
    client_secret: &str,
    current_refresh_token: &str,
) -> Result<TokenSet, AppError> {
    let form = [
        ("grant_type", "refresh_token"),
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("refresh_token", current_refresh_token),
    ];
    request_token(spec, base, &form, "refresh refused").await
}

pub async fn request_token(
    spec: &OAuthSpec,
    base: &str,
    form: &[(&str, &str)],
    refused: &str,
) -> Result<TokenSet, AppError> {
    request_token_with(spec, base, form, refused, TOKEN_BACKOFF).await
}

pub async fn request_token_with(
    spec: &OAuthSpec,
    base: &str,
    form: &[(&str, &str)],
    refused: &str,
    backoff: Duration,
) -> Result<TokenSet, AppError> {
    let http = crate::utils::http::client(base, Duration::from_secs(30))?;
    let url = spec.token_url(base);
    let mut fields: Vec<(&str, &str)> = spec.token_params.to_vec();
    fields.extend_from_slice(form);
    let basic = take_basic_credentials(spec, &mut fields);
    let mut attempt = 0u32;
    loop {
        let mut request = http.post(&url).form(&fields);
        if let Some((id, secret)) = &basic {
            request = request.basic_auth(id, Some(secret));
        }
        let reply = request.send().await?;
        let status = reply.status().as_u16();
        let text = reply.text().await?;
        match (spec.parse_token)(status, &text, refused) {
            Err(AppError::RateLimit(_)) if attempt + 1 < TOKEN_ATTEMPTS => {
                tokio::time::sleep(backoff * (1u32 << attempt)).await;
                attempt += 1;
            }
            outcome => return outcome,
        }
    }
}

fn take_basic_credentials(
    spec: &OAuthSpec,
    fields: &mut Vec<(&str, &str)>,
) -> Option<(String, String)> {
    if spec.client_auth != ClientAuth::Basic {
        return None;
    }
    let find = |fields: &[(&str, &str)], key: &str| {
        fields
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| (*v).to_string())
    };
    let id = find(fields, "client_id")?;
    let secret = find(fields, "client_secret")?;
    fields.retain(|(k, _)| *k != "client_id" && *k != "client_secret");
    Some((id, secret))
}

pub(crate) fn unreadable(text: &str) -> AppError {
    log::debug!(
        "[oauth] reponse de jeton illisible : {}",
        truncate_chars(&redact(text), 200)
    );
    AppError::Internal(format!("Token JSON illisible ({} octets)", text.len()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const LEAK: &str = "FUITE";

    fn ok_body() -> String {
        format!(
            r#"{{"access_token":"AT-{LEAK}","refresh_token":"RT-{LEAK}","expires_in":7200,"token_type":"Bearer","user_id":"u-9"}}"#
        )
    }

    #[test]
    fn a_standard_reply_becomes_a_token_set() {
        let set = standard_json(200, &ok_body(), "refresh refused").expect("jeton");
        assert_eq!(set.access_token, format!("AT-{LEAK}"));
        assert_eq!(set.refresh_token, format!("RT-{LEAK}"));
        assert_eq!(set.expires_in, 7200);
        assert_eq!(set.token_type, "Bearer");
        assert_eq!(set.user_ref, "u-9");
    }

    #[test]
    fn a_reply_without_refresh_token_or_expiry_gets_safe_defaults() {
        let set = standard_json(200, r#"{"access_token":"A"}"#, "refresh refused").expect("jeton");
        assert_eq!(set.refresh_token, "");
        assert_eq!(set.expires_in, 3600);
        assert_eq!(set.token_type, "Bearer");
        assert_eq!(set.user_ref, "");
    }

    #[test]
    fn a_refusal_names_the_error_code_and_never_the_body() {
        let body = format!(r#"{{"error":"invalid_grant","refresh_token":"RT-{LEAK}"}}"#);
        for status in [400u16, 401] {
            let err = standard_json(status, &body, "refresh refused").unwrap_err();
            assert_eq!(err.kind(), "Unauthorized", "{status}");
            let shown = err.to_string();
            assert!(shown.contains("invalid_grant"), "{shown}");
            assert!(shown.contains("refresh refused"), "{shown}");
            assert!(!shown.contains(LEAK), "{shown}");
        }
    }

    #[test]
    fn a_429_is_a_rate_limit_and_another_error_status_is_internal() {
        let limited = standard_json(429, "{}", "x").unwrap_err();
        assert_eq!(limited.kind(), "RateLimit");
        let broken = standard_json(500, "boom RT-FUITE", "x").unwrap_err();
        assert_eq!(broken.kind(), "Internal");
        assert!(!broken.to_string().contains(LEAK), "{broken}");
    }

    #[test]
    fn an_unreadable_success_names_the_size_not_the_body() {
        let text = format!("<html>RT-{LEAK}</html>");
        let err = standard_json(200, &text, "x").unwrap_err();
        assert_eq!(err.kind(), "Internal");
        let shown = err.to_string();
        assert!(!shown.contains(LEAK), "{shown}");
        assert!(shown.contains(&format!("{} octets", text.len())), "{shown}");
    }
}
