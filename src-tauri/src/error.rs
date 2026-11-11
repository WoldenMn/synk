use crate::utils::redact::redact;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Erreur base de donnees : {0}")]
    Db(sqlx::Error),

    #[error("Erreur de validation : {0}")]
    Validation(String),

    #[error("Ressource introuvable : {entity} #{id}")]
    NotFound { entity: &'static str, id: i64 },

    #[error("Ressource introuvable : {entity} {key}")]
    NotFoundByKey { entity: &'static str, key: String },

    #[error("JSON invalide : {0}")]
    Json(#[from] serde_json::Error),

    #[error("Acces refuse : {0}")]
    Forbidden(String),

    #[error("Erreur interne : {0}")]
    Internal(String),

    #[error("Erreur reseau : {0}")]
    Http(#[from] reqwest::Error),

    #[error("Acces expire : {0}")]
    Unauthorized(String),

    #[error("Limite atteinte : {0}")]
    RateLimit(String),

    #[error("Erreur de chiffrement : {0}")]
    Crypto(String),
}

impl AppError {
    pub fn kind(&self) -> &'static str {
        match self {
            AppError::Db(_) => "Db",
            AppError::Validation(_) => "Validation",
            AppError::NotFound { .. } => "NotFound",
            AppError::NotFoundByKey { .. } => "NotFound",
            AppError::Json(_) => "Json",
            AppError::Forbidden(_) => "Forbidden",
            AppError::Internal(_) => "Internal",
            AppError::Http(_) => "Http",
            AppError::Unauthorized(_) => "Unauthorized",
            AppError::RateLimit(_) => "RateLimit",
            AppError::Crypto(_) => "Crypto",
        }
    }

    pub fn detail(&self) -> String {
        match self {
            AppError::Db(e) => e.to_string(),
            AppError::Json(e) => e.to_string(),
            AppError::Http(e) => redact(&e.to_string()),
            AppError::NotFound { entity, id } => format!("{entity} #{id}"),
            AppError::NotFoundByKey { entity, key } => format!("{entity} {key}"),
            AppError::Validation(m)
            | AppError::Forbidden(m)
            | AppError::Internal(m)
            | AppError::Unauthorized(m)
            | AppError::RateLimit(m)
            | AppError::Crypto(m) => m.clone(),
        }
    }

    pub fn into_event_payload(self) -> serde_json::Value {
        serde_json::to_value(self.redacted()).unwrap_or_else(
            |_| serde_json::json!({"kind": "Internal", "message": "", "detail": ""}),
        )
    }

    pub fn redacted(self) -> AppError {
        match self {
            AppError::Validation(m) => AppError::Validation(redact(&m)),
            AppError::Forbidden(m) => AppError::Forbidden(redact(&m)),
            AppError::Internal(m) => AppError::Internal(redact(&m)),
            AppError::Unauthorized(m) => AppError::Unauthorized(redact(&m)),
            AppError::RateLimit(m) => AppError::RateLimit(redact(&m)),
            AppError::Crypto(m) => AppError::Crypto(redact(&m)),
            AppError::NotFound { .. } => self,
            AppError::NotFoundByKey { entity, key } => AppError::NotFoundByKey {
                entity,
                key: redact(&key),
            },
            AppError::Db(_) | AppError::Json(_) | AppError::Http(_) => {
                let text = self.to_string();
                let clean = redact(&text);
                if clean == text {
                    self
                } else {
                    AppError::Internal(clean)
                }
            }
        }
    }

    fn message(&self) -> String {
        match self {
            AppError::Http(_) => redact(&self.to_string()),
            _ => self.to_string(),
        }
    }
}

impl serde::Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("AppError", 3)?;
        s.serialize_field("kind", self.kind())?;
        s.serialize_field("message", &self.message())?;
        s.serialize_field("detail", &self.detail())?;
        s.end()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        if crate::db::gate::active() && crate::db::gate::is_busy(&e) {
            return AppError::Validation(crate::db::gate::REFUSAL.into());
        }
        AppError::Db(e)
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}

impl From<tauri::Error> for AppError {
    fn from(e: tauri::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}

impl From<csv::Error> for AppError {
    fn from(e: csv::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn json_of(err: &AppError) -> serde_json::Value {
        serde_json::to_value(err).unwrap()
    }

    #[test]
    fn serializes_kind_message_and_detail() {
        let json = json_of(&AppError::Validation("valeur hors bornes".to_string()));
        assert_eq!(json["kind"], "Validation");
        assert_eq!(json["detail"], "valeur hors bornes");
        assert_eq!(json["message"], "Erreur de validation : valeur hors bornes");
    }

    #[test]
    fn not_found_detail_names_the_entity() {
        let json = json_of(&AppError::NotFound {
            entity: "lab_results",
            id: 7,
        });
        assert_eq!(json["kind"], "NotFound");
        assert_eq!(json["detail"], "lab_results #7");
    }

    #[test]
    fn a_missing_key_is_a_not_found_that_names_the_key() {
        let json = json_of(&AppError::NotFoundByKey {
            entity: "metrique",
            key: "inconnue".into(),
        });
        assert_eq!(json["kind"], "NotFound");
        assert_eq!(json["detail"], "metrique inconnue");
    }

    #[test]
    fn every_variant_has_its_kind() {
        let json_err = serde_json::from_str::<i32>("x").unwrap_err();
        let cases: Vec<(AppError, &str)> = vec![
            (AppError::Db(sqlx::Error::RowNotFound), "Db"),
            (AppError::Validation("v".into()), "Validation"),
            (AppError::NotFound { entity: "e", id: 1 }, "NotFound"),
            (
                AppError::NotFoundByKey {
                    entity: "e",
                    key: "k".into(),
                },
                "NotFound",
            ),
            (AppError::Json(json_err), "Json"),
            (AppError::Forbidden("f".into()), "Forbidden"),
            (AppError::Internal("i".into()), "Internal"),
            (AppError::Unauthorized("u".into()), "Unauthorized"),
            (AppError::RateLimit("r".into()), "RateLimit"),
            (AppError::Crypto("c".into()), "Crypto"),
        ];
        for (err, kind) in cases {
            assert_eq!(err.kind(), kind);
            assert_eq!(json_of(&err)["kind"], kind);
        }
    }

    #[test]
    fn db_detail_is_the_source_text() {
        let err = AppError::Db(sqlx::Error::RowNotFound);
        assert_eq!(err.detail(), sqlx::Error::RowNotFound.to_string());
    }

    #[test]
    fn crypto_message_keeps_its_prefix() {
        let json = json_of(&AppError::Crypto("ciphertext tag mismatch".to_string()));
        assert_eq!(
            json["message"],
            "Erreur de chiffrement : ciphertext tag mismatch"
        );
    }

    #[test]
    fn redacted_masks_secrets_and_keeps_the_variant() {
        let err = AppError::Unauthorized("rappel ?code=abc123&state=x".to_string());
        let red = err.redacted();
        assert_eq!(red.kind(), "Unauthorized");
        assert!(!red.detail().contains("abc123"), "{}", red.detail());
        assert!(red.detail().contains("[masque]"));
    }

    #[test]
    fn redacted_leaves_a_clean_db_error_typed() {
        let red = AppError::Db(sqlx::Error::RowNotFound).redacted();
        assert_eq!(red.kind(), "Db");
    }

    #[test]
    fn event_payload_is_the_redacted_typed_error() {
        let payload =
            AppError::Unauthorized("echange refuse : {\"refresh_token\":\"RT-FUITE\"}".to_string())
                .into_event_payload();
        assert_eq!(payload["kind"], "Unauthorized");
        let text = payload.to_string();
        assert!(!text.contains("RT-FUITE"), "{text}");
        assert!(text.contains("[masque]"), "{text}");
    }
}
