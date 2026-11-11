use crate::common::{get_pair, set_pair};
use synk_lib::crypto::CryptoVault;

fn vault() -> CryptoVault {
    CryptoVault::from_key([0x11; 32])
}

#[tokio::test]
async fn empty_by_default() {
    let pool = crate::common::memory_pool().await;
    let v = vault();
    let (id, secret) = get_pair(&pool, &v).await.unwrap();
    assert_eq!(id.as_str(), "");
    assert_eq!(secret.as_str(), "");
}

#[tokio::test]
async fn set_and_get_roundtrip() {
    let pool = crate::common::memory_pool().await;
    let v = vault();
    set_pair(&pool, &v, "my_id", "my_secret").await.unwrap();
    let (id, secret) = get_pair(&pool, &v).await.unwrap();
    assert_eq!(id.as_str(), "my_id");
    assert_eq!(secret.as_str(), "my_secret");
}

#[tokio::test]
async fn set_overwrites_previous() {
    let pool = crate::common::memory_pool().await;
    let v = vault();
    set_pair(&pool, &v, "id1", "s1").await.unwrap();
    set_pair(&pool, &v, "id2", "s2").await.unwrap();
    let (id, secret) = get_pair(&pool, &v).await.unwrap();
    assert_eq!(id.as_str(), "id2");
    assert_eq!(secret.as_str(), "s2");
}

#[tokio::test]
async fn missing_client_id_error_is_french_and_points_to_settings() {
    let pool = crate::common::memory_pool().await;
    let v = vault();
    let err = synk_lib::sources::withings::client::client_id(&pool, &v)
        .await
        .unwrap_err();
    assert_eq!(err.kind(), "Validation", "obtenu : {err}");
    assert!(
        err.detail().starts_with("Identifiants Withings manquants"),
        "obtenu : {err}"
    );
    assert!(
        err.to_string().contains("Paramètres > Withings"),
        "obtenu : {err}"
    );
}

#[tokio::test]
async fn missing_client_secret_error_is_french_and_points_to_settings() {
    let pool = crate::common::memory_pool().await;
    let v = vault();
    let err = synk_lib::sources::withings::client::client_secret(&pool, &v)
        .await
        .unwrap_err();
    assert_eq!(err.kind(), "Validation", "obtenu : {err}");
    assert!(
        err.detail().starts_with("Identifiants Withings manquants"),
        "obtenu : {err}"
    );
    assert!(
        err.to_string().contains("Paramètres > Withings"),
        "obtenu : {err}"
    );
}
