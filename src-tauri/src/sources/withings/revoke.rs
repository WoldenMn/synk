use serde::Deserialize;

use super::client::{envelope_status, API_BASE};
use crate::error::AppError;
use crate::sources::oauth::revoke;
use crate::sources::{store, RevokeContext, SourceId};
use crate::utils::hmac::hmac_sha256_hex;

#[derive(Deserialize)]
struct Nonce {
    body: NonceBody,
}

#[derive(Deserialize)]
struct NonceBody {
    nonce: String,
}

async fn text_of(request: reqwest::RequestBuilder) -> Result<String, AppError> {
    let reply = request
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Withings : revocation impossible ({e})")))?;
    reply
        .text()
        .await
        .map_err(|e| AppError::Internal(format!("Withings : reponse illisible ({e})")))
}

fn refused(step: &str, status: Option<i64>) -> AppError {
    AppError::Internal(format!(
        "Withings : la marque a refuse la revocation de l'acces ({step}, code {})",
        status.map_or("illisible".to_string(), |s| s.to_string())
    ))
}

pub async fn run(base: Option<&str>, ctx: &RevokeContext<'_>) -> Result<bool, AppError> {
    let base = base.unwrap_or(API_BASE);
    let Some(stored) = store::load_auth(ctx.pool, ctx.vault.as_ref(), SourceId::WITHINGS).await?
    else {
        return Ok(false);
    };
    if stored.user_ref.is_empty() {
        return Err(AppError::Validation(
            "Withings : identifiant d'utilisateur absent, retirez l'acces dans votre compte Withings"
                .into(),
        ));
    }
    let (client_id, client_secret) =
        store::require_credentials(ctx.pool, ctx.vault.as_ref(), SourceId::WITHINGS).await?;
    let http = revoke::client(base)?;
    let timestamp = chrono::Utc::now().timestamp().to_string();
    let signature = hmac_sha256_hex(
        client_secret.as_bytes(),
        format!("getnonce,{},{timestamp}", client_id.as_str()).as_bytes(),
    );
    let text = text_of(http.post(format!("{base}/v2/signature")).form(&[
        ("action", "getnonce"),
        ("client_id", client_id.as_str()),
        ("timestamp", timestamp.as_str()),
        ("signature", signature.as_str()),
    ]))
    .await?;
    let nonce = match envelope_status(&text) {
        Some(0) => {
            serde_json::from_str::<Nonce>(&text)
                .map_err(|_| refused("nonce", None))?
                .body
                .nonce
        }
        other => return Err(refused("nonce", other)),
    };
    let signature = hmac_sha256_hex(
        client_secret.as_bytes(),
        format!("revoke,{},{nonce}", client_id.as_str()).as_bytes(),
    );
    let text = text_of(http.post(format!("{base}/v2/oauth2")).form(&[
        ("action", "revoke"),
        ("client_id", client_id.as_str()),
        ("nonce", nonce.as_str()),
        ("signature", signature.as_str()),
        ("userid", stored.user_ref.as_str()),
    ]))
    .await?;
    match envelope_status(&text) {
        Some(0) => Ok(true),
        other => Err(refused("revocation", other)),
    }
}
