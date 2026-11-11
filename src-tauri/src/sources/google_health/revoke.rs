use crate::error::AppError;
use crate::sources::oauth::revoke;
use crate::sources::{store, RevokeContext, SourceId};

pub async fn run(token_base: &str, ctx: &RevokeContext<'_>) -> Result<bool, AppError> {
    let Some(stored) =
        store::load_auth(ctx.pool, ctx.vault.as_ref(), SourceId::GOOGLE_HEALTH).await?
    else {
        return Ok(false);
    };
    let token = if stored.refresh_token.is_empty() {
        stored.access_token.as_str()
    } else {
        stored.refresh_token.as_str()
    };
    let request = revoke::client(token_base)?
        .post(format!("{token_base}/revoke"))
        .form(&[("token", token)]);
    revoke::send(SourceId::GOOGLE_HEALTH, request).await?;
    Ok(true)
}
