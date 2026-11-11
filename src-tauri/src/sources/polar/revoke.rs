use crate::error::AppError;
use crate::sources::oauth::revoke;
use crate::sources::{store, RevokeContext, SourceId};

pub async fn run(base: &str, ctx: &RevokeContext<'_>) -> Result<bool, AppError> {
    let Some(stored) = store::load_auth(ctx.pool, ctx.vault.as_ref(), SourceId::POLAR).await?
    else {
        return Ok(false);
    };
    if stored.user_ref.is_empty() {
        return Err(AppError::Internal(
            "Polar : identifiant d'utilisateur absent, retire l'accès depuis ton compte Polar Flow"
                .to_string(),
        ));
    }
    let url = format!("{base}/v3/users/{}", urlencoding::encode(&stored.user_ref));
    let request = revoke::client(base)?
        .delete(url)
        .bearer_auth(stored.access_token.as_str());
    revoke::send(SourceId::POLAR, request).await?;
    Ok(true)
}
