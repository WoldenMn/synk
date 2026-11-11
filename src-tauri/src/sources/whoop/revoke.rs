use super::spec::OAUTH;
use crate::error::AppError;
use crate::sources::oauth::{self, revoke};
use crate::sources::{store, RevokeContext, SourceId};

pub async fn run(base: &str, ctx: &RevokeContext<'_>) -> Result<bool, AppError> {
    let Some(stored) = store::load_auth(ctx.pool, ctx.vault.as_ref(), SourceId::WHOOP).await?
    else {
        return Ok(false);
    };
    let mut token = stored.access_token.to_string();
    if stored.expires_at < chrono::Utc::now().timestamp() + 300 {
        token = oauth::refresh_locked(&OAUTH, base, ctx.pool, ctx.vault.as_ref(), &token)
            .await?
            .to_string();
    }
    let request = revoke::client(base)?
        .delete(format!("{base}/developer/v2/user/access"))
        .bearer_auth(token);
    revoke::send(SourceId::WHOOP, request).await?;
    Ok(true)
}
