use crate::error::AppError;
use crate::sources::oauth::{ClientAuth, OAuthSpec, TokenSet};
use crate::sources::SourceId;
use crate::utils::redact::redact;
use crate::utils::text::truncate_chars;

use super::client::{envelope_status, API_BASE};
use super::types::{TokenBody, WithingsResponse};

pub static OAUTH: OAuthSpec = OAuthSpec {
    id: SourceId::WITHINGS,
    authorize_url: "https://account.withings.com/oauth2_user/authorize2",
    api_base: API_BASE,
    token_path: "/v2/oauth2",
    scopes: "user.info,user.metrics,user.activity",
    pkce: false,
    redirect_path: Some("/callback"),
    token_params: &[("action", "requesttoken")],
    authorize_params: &[],
    refresh_lifetime_secs: None,
    parse_token: parse_envelope,
    client_auth: ClientAuth::Body,
};

pub fn parse_envelope(_status: u16, text: &str, refused: &str) -> Result<TokenSet, AppError> {
    match envelope_status(text) {
        Some(0) => {}
        Some(401) => {
            return Err(AppError::Unauthorized(format!(
                "Withings code 401: {refused}"
            )))
        }
        Some(601) => {
            return Err(AppError::RateLimit(
                "Withings code 601: rate limit".to_string(),
            ))
        }
        Some(other) => {
            return Err(AppError::Internal(format!(
                "Withings code {other}: token request"
            )))
        }
        None => {
            log::debug!(
                "[withings] reponse de jeton illisible : {}",
                truncate_chars(&redact(text), 200)
            );
            return Err(AppError::Internal(format!(
                "Token JSON illisible ({} octets)",
                text.len()
            )));
        }
    }
    let parsed: WithingsResponse<TokenBody> = serde_json::from_str(text).map_err(|e| {
        log::debug!(
            "[withings] reponse de jeton non decodee : {}",
            truncate_chars(&redact(text), 200)
        );
        AppError::Internal(format!(
            "Token JSON parse error: {:?} ligne {} colonne {} ({} octets)",
            e.classify(),
            e.line(),
            e.column(),
            text.len()
        ))
    })?;
    Ok(parsed.body.to_set())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_withings_token_url_and_form_are_unchanged() {
        assert_eq!(
            OAUTH.token_url("https://wbsapi.withings.net"),
            "https://wbsapi.withings.net/v2/oauth2"
        );
        assert_eq!(OAUTH.token_params, &[("action", "requesttoken")]);
    }

    #[test]
    fn an_envelope_reply_becomes_a_token_set() {
        let text = r#"{"status":0,"body":{"userid":"4242","access_token":"A","refresh_token":"R","expires_in":10800,"token_type":"Bearer","scope":"user.info"}}"#;
        let set = parse_envelope(200, text, "x").expect("jeton");
        assert_eq!(
            (set.access_token.as_str(), set.refresh_token.as_str()),
            ("A", "R")
        );
        assert_eq!(set.user_ref, "4242");
        assert_eq!(set.expires_in, 10800);
    }

    #[test]
    fn the_envelope_status_decides_even_on_http_200() {
        assert_eq!(
            parse_envelope(200, r#"{"status":401}"#, "refresh refused")
                .unwrap_err()
                .kind(),
            "Unauthorized"
        );
        assert_eq!(
            parse_envelope(200, r#"{"status":601}"#, "x")
                .unwrap_err()
                .kind(),
            "RateLimit"
        );
        assert_eq!(
            parse_envelope(200, r#"{"status":503}"#, "x")
                .unwrap_err()
                .kind(),
            "Internal"
        );
    }
}
