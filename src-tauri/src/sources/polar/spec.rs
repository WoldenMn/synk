use serde::Deserialize;

use crate::error::AppError;
use crate::sources::oauth::token::{standard_json, unreadable};
use crate::sources::oauth::{ClientAuth, OAuthSpec, TokenSet};
use crate::sources::SourceId;

pub const API_BASE: &str = "https://www.polaraccesslink.com";
pub const TOKEN_BASE: &str = "https://polarremote.com";
pub const DEVELOPER_URL: &str = "https://admin.polaraccesslink.com/";

pub static OAUTH: OAuthSpec = OAuthSpec {
    id: SourceId::POLAR,
    authorize_url: "https://flow.polar.com/oauth2/authorization",
    api_base: TOKEN_BASE,
    token_path: "/v2/oauth2/token",
    scopes: "accesslink.read_all",
    pkce: false,
    redirect_path: None,
    token_params: &[],
    authorize_params: &[],
    refresh_lifetime_secs: None,
    parse_token,
    client_auth: ClientAuth::Basic,
};

#[derive(Deserialize)]
struct TokenReply {
    access_token: String,
    #[serde(default = "one_year")]
    expires_in: i64,
    #[serde(default)]
    token_type: Option<String>,
    #[serde(default)]
    x_user_id: Option<serde_json::Value>,
}

fn one_year() -> i64 {
    31_535_999
}

pub fn parse_token(status: u16, body: &str, refused: &str) -> Result<TokenSet, AppError> {
    if !(200..300).contains(&status) {
        return standard_json(status, body, refused);
    }
    let reply: TokenReply = serde_json::from_str(body).map_err(|_| unreadable(body))?;
    Ok(TokenSet {
        access_token: reply.access_token,
        refresh_token: String::new(),
        expires_in: reply.expires_in,
        token_type: reply.token_type.unwrap_or_else(|| "bearer".to_string()),
        user_ref: match reply.x_user_id {
            Some(serde_json::Value::String(s)) => s,
            Some(other) => other.to_string(),
            None => String::new(),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_authorization_url_asks_the_single_scope_and_the_exact_registered_callback() {
        let url = OAUTH.authorization_url("client-test", "etat-1", None);
        assert!(
            url.starts_with("https://flow.polar.com/oauth2/authorization?response_type=code"),
            "{url}"
        );
        assert!(url.contains("&scope=accesslink.read_all"), "{url}");
        assert!(
            url.contains("&redirect_uri=http%3A%2F%2Flocalhost%3A17483%2Fcallback%2Fpolar"),
            "{url}"
        );
        assert!(!url.contains("code_challenge"), "{url}");
        assert!(!OAUTH.pkce);
    }

    #[test]
    fn the_token_endpoint_lives_on_its_own_host_and_wants_basic_authentication() {
        assert_eq!(
            OAUTH.token_url(OAUTH.api_base),
            "https://polarremote.com/v2/oauth2/token"
        );
        assert_eq!(OAUTH.client_auth, ClientAuth::Basic);
        assert_eq!(
            OAUTH.redirect_uri(),
            "http://localhost:17483/callback/polar"
        );
    }

    #[test]
    fn a_token_reply_gives_the_access_token_the_user_id_and_no_refresh_token() {
        let set = parse_token(
            200,
            r#"{"access_token":"AT-1","token_type":"bearer","expires_in":31535999,"x_user_id":10579}"#,
            "refused",
        )
        .expect("jeton");
        assert_eq!(set.access_token, "AT-1");
        assert_eq!(set.refresh_token, "");
        assert_eq!(set.expires_in, 31_535_999);
        assert_eq!(set.user_ref, "10579");
    }

    #[test]
    fn a_refused_code_is_unauthorized_and_never_echoes_the_body() {
        let err = parse_token(
            400,
            r#"{"error":"invalid_grant","secret":"FUITE"}"#,
            "refused",
        )
        .unwrap_err();
        assert_eq!(err.kind(), "Unauthorized");
        assert!(err.to_string().contains("invalid_grant"), "{err}");
        assert!(!err.to_string().contains("FUITE"), "{err}");
    }

    #[test]
    fn an_unreadable_success_names_the_size_only() {
        let err = parse_token(200, "<html>FUITE</html>", "x").unwrap_err();
        assert_eq!(err.kind(), "Internal");
        assert!(!err.to_string().contains("FUITE"), "{err}");
    }
}
