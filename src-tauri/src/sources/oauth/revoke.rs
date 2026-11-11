use std::time::Duration;

use crate::error::AppError;
use crate::sources::{label_of, SourceId};
use crate::utils::redact::redact;

const REVOKE_TIMEOUT: Duration = Duration::from_secs(15);

pub fn client(base: &str) -> Result<reqwest::Client, AppError> {
    crate::utils::http::client(base, REVOKE_TIMEOUT)
}

pub async fn send(source: SourceId, request: reqwest::RequestBuilder) -> Result<(), AppError> {
    let label = label_of(source);
    match request.send().await {
        Ok(reply) if reply.status().is_success() => Ok(()),
        Ok(reply) => Err(AppError::Internal(format!(
            "{label} : la marque a refusé la révocation de l'accès (HTTP {})",
            reply.status().as_u16()
        ))),
        Err(e) => Err(AppError::Internal(format!(
            "{label} : révocation impossible, la marque n'a pas répondu ({})",
            redact(&e.without_url().to_string())
        ))),
    }
}
