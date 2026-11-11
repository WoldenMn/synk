use crate::sources::oauth::token::standard_json;
use crate::sources::oauth::{ClientAuth, OAuthSpec};
use crate::sources::SourceId;

pub const API_BASE: &str = "https://api.prod.whoop.com";
pub const DEVELOPER_URL: &str = "https://developer-dashboard.whoop.com/";

pub static OAUTH: OAuthSpec = OAuthSpec {
    id: SourceId::WHOOP,
    authorize_url: "https://api.prod.whoop.com/oauth/oauth2/auth",
    api_base: API_BASE,
    token_path: "/oauth/oauth2/token",
    scopes: "offline read:recovery read:sleep read:cycles read:workout",
    pkce: false,
    redirect_path: None,
    token_params: &[],
    authorize_params: &[],
    refresh_lifetime_secs: None,
    parse_token: standard_json,
    client_auth: ClientAuth::Body,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::oauth::flow::generate_state;

    #[test]
    fn the_authorization_url_asks_offline_and_the_four_read_scopes_without_pkce() {
        let url = OAUTH.authorization_url("client-test", "etat-de-test", None);
        assert!(
            url.starts_with("https://api.prod.whoop.com/oauth/oauth2/auth?response_type=code"),
            "{url}"
        );
        assert!(
            url.contains(
                "&scope=offline%20read%3Arecovery%20read%3Asleep%20read%3Acycles%20read%3Aworkout"
            ),
            "{url}"
        );
        assert!(
            url.contains("&redirect_uri=http%3A%2F%2Flocalhost%3A17483%2Fcallback%2Fwhoop"),
            "{url}"
        );
        assert!(!url.contains("code_challenge"), "{url}");
        assert!(!OAUTH.pkce);
    }

    #[test]
    fn the_token_endpoint_is_on_the_api_host_and_credentials_travel_in_the_body() {
        assert_eq!(
            OAUTH.token_url(API_BASE),
            "https://api.prod.whoop.com/oauth/oauth2/token"
        );
        assert_eq!(OAUTH.client_auth, ClientAuth::Body);
        assert_eq!(
            OAUTH.redirect_uri(),
            "http://localhost:17483/callback/whoop"
        );
    }

    #[test]
    fn the_state_is_long_enough_for_whoop_which_wants_eight_characters_at_least() {
        assert!(generate_state().expect("etat").len() >= 8);
    }

    #[test]
    fn the_offline_scope_is_requested_or_no_refresh_token_would_ever_be_issued() {
        assert!(OAUTH.scopes.split(' ').any(|s| s == "offline"));
    }
}
