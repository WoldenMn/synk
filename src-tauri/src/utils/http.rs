use std::time::Duration;

use crate::error::AppError;

pub fn client(base: &str, timeout: Duration) -> Result<reqwest::Client, AppError> {
    reqwest::Client::builder()
        .timeout(timeout)
        .redirect(reqwest::redirect::Policy::none())
        .https_only(base.starts_with("https://"))
        .build()
        .map_err(|e| AppError::Internal(format!("reqwest Client build failed: {e}")))
}
