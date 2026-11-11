use crate::sources::oauth::token::standard_json;
use crate::sources::oauth::{ClientAuth, OAuthSpec};
use crate::sources::SourceId;

pub const API_BASE: &str = "https://api.ouraring.com";
pub const DEVELOPER_URL: &str = "https://cloud.ouraring.com/oauth/applications";

pub static OAUTH: OAuthSpec = OAuthSpec {
    id: SourceId::OURA,
    authorize_url: "https://cloud.ouraring.com/oauth/authorize",
    api_base: API_BASE,
    token_path: "/oauth/token",
    scopes: "daily spo2 workout",
    pkce: true,
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

    #[test]
    fn the_authorization_url_asks_for_pkce_and_the_exact_registered_callback() {
        let url = OAUTH.authorization_url("client-test", "etat-1", Some("defi-1"));
        assert!(
            url.starts_with("https://cloud.ouraring.com/oauth/authorize?response_type=code"),
            "{url}"
        );
        assert!(url.contains("&scope=daily%20spo2%20workout"), "{url}");
        assert!(
            url.contains("&redirect_uri=http%3A%2F%2Flocalhost%3A17483%2Fcallback%2Foura"),
            "{url}"
        );
        assert!(url.contains("&code_challenge=defi-1"), "{url}");
        assert!(url.ends_with("&code_challenge_method=S256"), "{url}");
    }

    #[test]
    fn the_token_endpoint_lives_on_the_api_host_not_the_cloud_host() {
        assert_eq!(
            OAUTH.token_url(API_BASE),
            "https://api.ouraring.com/oauth/token"
        );
        assert_eq!(OAUTH.redirect_uri(), "http://localhost:17483/callback/oura");
    }

    #[test]
    fn only_the_scopes_the_sync_reads_are_requested() {
        let scopes: Vec<&str> = OAUTH.scopes.split(' ').collect();
        assert_eq!(scopes, vec!["daily", "spo2", "workout"]);
    }
}
