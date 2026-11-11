use crate::sources::oauth::token::standard_json;
use crate::sources::oauth::{ClientAuth, OAuthSpec};
use crate::sources::SourceId;

pub const TOKEN_BASE: &str = "https://oauth2.googleapis.com";
pub const HEALTH_BASE: &str = "https://health.googleapis.com";
pub const DEVELOPER_URL: &str =
    "https://console.developers.google.com/apis/library/health.googleapis.com";
pub const TESTING_REFRESH_LIFETIME: i64 = 7 * 86_400;

pub static OAUTH: OAuthSpec = OAuthSpec {
    id: SourceId::GOOGLE_HEALTH,
    authorize_url: "https://accounts.google.com/o/oauth2/v2/auth",
    api_base: TOKEN_BASE,
    token_path: "/token",
    scopes: "https://www.googleapis.com/auth/googlehealth.sleep.readonly https://www.googleapis.com/auth/googlehealth.activity_and_fitness.readonly https://www.googleapis.com/auth/googlehealth.health_metrics_and_measurements.readonly",
    pkce: true,
    redirect_path: None,
    token_params: &[],
    authorize_params: &[("access_type", "offline"), ("prompt", "consent")],
    refresh_lifetime_secs: Some(TESTING_REFRESH_LIFETIME),
    parse_token: standard_json,
    client_auth: ClientAuth::Body,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::oauth::CALLBACK_ADDR;

    #[test]
    fn the_authorization_url_asks_offline_access_pkce_and_the_exact_callback() {
        let url = OAUTH.authorization_url("client-test", "etat-1", Some("defi-1"));
        assert!(
            url.starts_with("https://accounts.google.com/o/oauth2/v2/auth?response_type=code"),
            "{url}"
        );
        assert!(url.contains("&access_type=offline"), "{url}");
        assert!(url.contains("&prompt=consent"), "{url}");
        assert!(url.contains("&code_challenge=defi-1"), "{url}");
        assert!(url.contains("&code_challenge_method=S256"), "{url}");
        assert!(
            url.contains("&redirect_uri=http%3A%2F%2Flocalhost%3A17483%2Fcallback%2Fgoogle_health"),
            "{url}"
        );
    }

    #[test]
    fn the_declared_callback_is_localhost_and_the_listener_is_ipv4_loopback_only() {
        assert_eq!(
            OAUTH.redirect_uri(),
            "http://localhost:17483/callback/google_health"
        );
        assert!(CALLBACK_ADDR.starts_with("127.0.0.1:"), "{CALLBACK_ADDR}");
    }

    #[test]
    fn only_the_three_read_only_scopes_are_requested() {
        let scopes: Vec<&str> = OAUTH.scopes.split(' ').collect();
        assert_eq!(
            scopes,
            vec![
                "https://www.googleapis.com/auth/googlehealth.sleep.readonly",
                "https://www.googleapis.com/auth/googlehealth.activity_and_fitness.readonly",
                "https://www.googleapis.com/auth/googlehealth.health_metrics_and_measurements.readonly",
            ]
        );
        assert!(scopes.iter().all(|s| s.ends_with(".readonly")));
    }

    #[test]
    fn the_token_endpoint_is_google_and_the_testing_lifetime_is_seven_days() {
        assert_eq!(
            OAUTH.token_url(TOKEN_BASE),
            "https://oauth2.googleapis.com/token"
        );
        assert_eq!(OAUTH.refresh_lifetime_secs, Some(604_800));
    }
}
