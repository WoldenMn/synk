use sqlx::SqlitePool;

use super::credentials::{client_id, client_secret};
use crate::error::AppError;
use crate::sources::oauth::token::refresh_token;
use crate::sources::oauth::TokenSet;
use crate::sources::withings::spec::OAUTH;

pub async fn refresh_token_at(
    base: &str,
    pool: &SqlitePool,
    current_refresh_token: &str,
    vault: &crate::crypto::CryptoVault,
) -> Result<TokenSet, AppError> {
    let cid = client_id(pool, vault).await?;
    let csecret = client_secret(pool, vault).await?;
    refresh_token(&OAUTH, base, &cid, &csecret, current_refresh_token).await
}
