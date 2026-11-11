use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct WithingsResponse<T> {
    pub status: i64,
    pub body: T,
}

#[derive(Deserialize)]
pub struct TokenBody {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub token_type: String,
    pub userid: serde_json::Value,
}

impl TokenBody {
    pub fn to_set(&self) -> crate::sources::oauth::TokenSet {
        crate::sources::oauth::TokenSet {
            access_token: self.access_token.clone(),
            refresh_token: self.refresh_token.clone(),
            expires_in: self.expires_in,
            token_type: self.token_type.clone(),
            user_ref: self
                .userid
                .as_str()
                .map(str::to_string)
                .unwrap_or_else(|| self.userid.to_string()),
        }
    }
}

impl std::fmt::Debug for TokenBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenBody")
            .field("access_token", &crate::utils::redact::MASK)
            .field("refresh_token", &crate::utils::redact::MASK)
            .field("expires_in", &self.expires_in)
            .field("token_type", &self.token_type)
            .field("userid", &self.userid)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::TokenBody;

    const LEAK: &str = "FUITE";

    #[test]
    fn debug_output_hides_both_tokens() {
        let body = TokenBody {
            access_token: format!("AT-{LEAK}"),
            refresh_token: format!("RT-{LEAK}"),
            expires_in: 10800,
            token_type: "Bearer".to_string(),
            userid: serde_json::json!(1),
        };
        let shown = format!("{body:?}");
        assert!(!shown.contains(LEAK), "{shown}");
        assert!(shown.contains("10800"), "{shown}");
    }
}
