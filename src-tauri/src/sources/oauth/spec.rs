use std::time::Duration;

use crate::error::AppError;
use crate::sources::SourceId;

pub const CALLBACK_PORT: u16 = 17483;
pub const CALLBACK_ADDR: &str = "127.0.0.1:17483";
pub const CALLBACK_ORIGIN: &str = "http://localhost:17483";
pub const AUTH_TIMEOUT: Duration = Duration::from_secs(120);

pub type TokenParser = fn(u16, &str, &str) -> Result<TokenSet, AppError>;

#[derive(Clone)]
pub struct TokenSet {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub token_type: String,
    pub user_ref: String,
}

impl TokenSet {
    pub fn wipe(&mut self) {
        use zeroize::Zeroize;
        self.access_token.zeroize();
        self.refresh_token.zeroize();
    }
}

impl Drop for TokenSet {
    fn drop(&mut self) {
        self.wipe();
    }
}

impl std::fmt::Debug for TokenSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenSet")
            .field("access_token", &crate::utils::redact::MASK)
            .field("refresh_token", &crate::utils::redact::MASK)
            .field("expires_in", &self.expires_in)
            .field("token_type", &self.token_type)
            .field("user_ref", &self.user_ref)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientAuth {
    Body,
    Basic,
}

#[derive(Debug)]
pub struct OAuthSpec {
    pub id: SourceId,
    pub authorize_url: &'static str,
    pub api_base: &'static str,
    pub token_path: &'static str,
    pub scopes: &'static str,
    pub pkce: bool,
    pub redirect_path: Option<&'static str>,
    pub token_params: &'static [(&'static str, &'static str)],
    pub authorize_params: &'static [(&'static str, &'static str)],
    pub refresh_lifetime_secs: Option<i64>,
    pub parse_token: TokenParser,
    pub client_auth: ClientAuth,
}

impl OAuthSpec {
    pub fn callback_path(&self) -> String {
        match self.redirect_path {
            Some(path) => path.to_string(),
            None => format!("/callback/{}", self.id.as_str()),
        }
    }

    pub fn redirect_uri(&self) -> String {
        format!("{CALLBACK_ORIGIN}{}", self.callback_path())
    }

    pub fn token_url(&self, base: &str) -> String {
        format!("{base}{}", self.token_path)
    }

    pub fn authorization_url(
        &self,
        client_id: &str,
        state: &str,
        code_challenge: Option<&str>,
    ) -> String {
        let mut url = format!(
            "{}?response_type=code&client_id={}&redirect_uri={}&state={}&scope={}",
            self.authorize_url,
            urlencoding::encode(client_id),
            urlencoding::encode(&self.redirect_uri()),
            urlencoding::encode(state),
            urlencoding::encode(self.scopes),
        );
        if let Some(challenge) = code_challenge {
            url.push_str("&code_challenge=");
            url.push_str(&urlencoding::encode(challenge));
            url.push_str("&code_challenge_method=S256");
        }
        for (key, value) in self.authorize_params {
            url.push('&');
            url.push_str(key);
            url.push('=');
            url.push_str(&urlencoding::encode(value));
        }
        url
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::oauth::token::standard_json;

    static PLAIN: OAuthSpec = OAuthSpec {
        id: SourceId::new("oura"),
        authorize_url: "https://cloud.example.test/oauth/authorize",
        api_base: "https://api.example.test",
        token_path: "/oauth/token",
        scopes: "daily heartrate",
        pkce: false,
        redirect_path: None,
        token_params: &[],
        authorize_params: &[],
        refresh_lifetime_secs: None,
        parse_token: standard_json,
        client_auth: ClientAuth::Body,
    };

    #[test]
    fn a_source_without_a_registered_path_gets_a_callback_named_after_it() {
        assert_eq!(PLAIN.callback_path(), "/callback/oura");
        assert_eq!(PLAIN.redirect_uri(), "http://localhost:17483/callback/oura");
    }

    #[test]
    fn the_authorization_url_encodes_every_value_and_adds_pkce_only_on_request() {
        let plain = PLAIN.authorization_url("cid&1", "etat/x", None);
        assert!(
            plain.starts_with("https://cloud.example.test/oauth/authorize?response_type=code"),
            "{plain}"
        );
        assert!(plain.contains("client_id=cid%261"), "{plain}");
        assert!(plain.contains("state=etat%2Fx"), "{plain}");
        assert!(plain.contains("scope=daily%20heartrate"), "{plain}");
        assert!(
            plain.contains("redirect_uri=http%3A%2F%2Flocalhost%3A17483%2Fcallback%2Foura"),
            "{plain}"
        );
        assert!(!plain.contains("code_challenge"), "{plain}");

        let pkce = PLAIN.authorization_url("cid", "s", Some("abc-_"));
        assert!(pkce.contains("&code_challenge=abc-_"), "{pkce}");
        assert!(pkce.ends_with("&code_challenge_method=S256"), "{pkce}");
    }

    #[test]
    fn extra_authorization_parameters_are_appended_encoded() {
        static EXTRA: OAuthSpec = OAuthSpec {
            authorize_params: &[("access_type", "offline"), ("prompt", "consent x")],
            ..PLAIN
        };
        let url = EXTRA.authorization_url("cid", "s", None);
        assert!(url.contains("&access_type=offline"), "{url}");
        assert!(url.contains("&prompt=consent%20x"), "{url}");
        assert!(!PLAIN
            .authorization_url("cid", "s", None)
            .contains("access_type"));
    }

    #[test]
    fn the_token_url_joins_the_base_and_the_path() {
        assert_eq!(
            PLAIN.token_url("http://127.0.0.1:9"),
            "http://127.0.0.1:9/oauth/token"
        );
    }

    #[test]
    fn a_token_set_hides_both_tokens_when_printed() {
        let set = TokenSet {
            access_token: "AT-FUITE".into(),
            refresh_token: "RT-FUITE".into(),
            expires_in: 3600,
            token_type: "Bearer".into(),
            user_ref: "u1".into(),
        };
        let shown = format!("{set:?}");
        assert!(!shown.contains("FUITE"), "{shown}");
        assert!(shown.contains("3600"), "{shown}");
    }

    #[test]
    fn wiping_a_token_set_clears_both_tokens() {
        let mut set = TokenSet {
            access_token: "AT-1".into(),
            refresh_token: "RT-1".into(),
            expires_in: 3600,
            token_type: "Bearer".into(),
            user_ref: "u1".into(),
        };
        set.wipe();
        assert!(set.access_token.is_empty() && set.refresh_token.is_empty());
        assert_eq!(set.user_ref, "u1");
    }
}
