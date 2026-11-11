use synk_lib::error::AppError;

#[test]
fn test_db_error_displays_with_prefix() {
    let sqlx_err = sqlx::Error::RowNotFound;
    let app_err: AppError = sqlx_err.into();
    let msg = app_err.to_string();
    assert!(
        msg.starts_with("Erreur base de donnees : "),
        "expected prefix 'Erreur base de donnees : ', got: {}",
        msg
    );
}

#[test]
fn test_validation_error_displays() {
    let err = AppError::Validation("Systolique hors plage".into());
    assert_eq!(
        err.to_string(),
        "Erreur de validation : Systolique hors plage"
    );
}

#[test]
fn test_not_found_error_displays_entity_and_id() {
    let err = AppError::NotFound {
        entity: "blood_pressure",
        id: 42,
    };
    assert_eq!(
        err.to_string(),
        "Ressource introuvable : blood_pressure #42"
    );
}

#[test]
fn test_app_error_crosses_the_ipc_as_kind_message_detail() {
    let err = AppError::Forbidden("source != manual".into());
    let json = serde_json::to_value(&err).unwrap();
    assert_eq!(
        json,
        serde_json::json!({
            "kind": "Forbidden",
            "message": "Acces refuse : source != manual",
            "detail": "source != manual"
        })
    );
}

#[test]
fn test_unauthorized_error_displays_with_prefix() {
    let err = synk_lib::error::AppError::Unauthorized("token expired".into());
    assert_eq!(err.to_string(), "Acces expire : token expired");
}

#[test]
fn test_rate_limit_error_displays_with_prefix() {
    let err = synk_lib::error::AppError::RateLimit("Withings code 601".into());
    assert_eq!(err.to_string(), "Limite atteinte : Withings code 601");
}
