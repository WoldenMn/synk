use base64::{engine::general_purpose::STANDARD, Engine as _};
use keyring_core::{Entry, Error as KeyringError};
use std::sync::OnceLock;
use zeroize::Zeroizing;

use crate::error::AppError;

const PROD_SERVICE: &str = "synk";
const DEV_SERVICE: &str = "synk-dev";
const KEY_USER: &str = "master_key";

pub(crate) fn service_for(identifier: &str) -> &'static str {
    if identifier == crate::boot::PROD_IDENTIFIER {
        PROD_SERVICE
    } else {
        DEV_SERVICE
    }
}

fn init_store() -> Result<(), AppError> {
    static STORE: OnceLock<Result<(), String>> = OnceLock::new();
    STORE
        .get_or_init(|| {
            let store = windows_native_keyring_store::Store::new().map_err(|e| e.to_string())?;
            keyring_core::set_default_store(store);
            Ok(())
        })
        .clone()
        .map_err(|e| AppError::Crypto(format!("keyring store: {e}")))
}

pub(crate) fn get_or_create_master_key(service: &str) -> Result<Zeroizing<[u8; 32]>, AppError> {
    init_store()?;
    let entry = Entry::new(service, KEY_USER)
        .map_err(|e| AppError::Crypto(format!("keyring init: {e}")))?;
    get_or_create_with_entry(&entry)
}

fn get_or_create_with_entry(entry: &Entry) -> Result<Zeroizing<[u8; 32]>, AppError> {
    match entry.get_password() {
        Ok(b64) => decode_key(&Zeroizing::new(b64)),
        Err(KeyringError::NoEntry) => {
            let key = generate_key()?;
            entry
                .set_password(&Zeroizing::new(STANDARD.encode(*key)))
                .map_err(|e| AppError::Crypto(format!("keyring set: {e}")))?;
            log::warn!("crypto: master key generated and stored in keyring (first run)");
            Ok(key)
        }
        Err(e) => Err(AppError::Crypto(format!("keyring get: {e}"))),
    }
}

fn generate_key() -> Result<Zeroizing<[u8; 32]>, AppError> {
    let mut key = Zeroizing::new([0u8; 32]);
    getrandom::fill(key.as_mut_slice())
        .map_err(|e| AppError::Crypto(format!("keyring rng: {e}")))?;
    Ok(key)
}

fn decode_key(b64: &str) -> Result<Zeroizing<[u8; 32]>, AppError> {
    let bytes = Zeroizing::new(
        STANDARD
            .decode(b64)
            .map_err(|e| AppError::Crypto(format!("key base64: {e}")))?,
    );
    if bytes.len() != 32 {
        return Err(AppError::Crypto(format!(
            "key wrong length: {}",
            bytes.len()
        )));
    }
    let mut out = Zeroizing::new([0u8; 32]);
    out.copy_from_slice(&bytes);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyring_core::api::CredentialStoreApi;

    fn mock_entry() -> Entry {
        keyring_core::mock::Store::new()
            .expect("magasin factice")
            .build("synk-test", KEY_USER, None)
            .expect("entree factice")
    }

    #[test]
    fn init_store_installs_the_windows_store() {
        init_store().expect("magasin");
        let store = keyring_core::get_default_store().expect("magasin par defaut");
        assert!(
            store.vendor().starts_with("Windows Credential Manager"),
            "{}",
            store.vendor()
        );
    }

    #[test]
    fn first_call_generates_and_second_returns_same() {
        let entry = mock_entry();
        let k1 = get_or_create_with_entry(&entry).expect("first");
        assert_eq!(k1.len(), 32);
        assert!(k1.iter().any(|&b| b != 0));
        let k2 = get_or_create_with_entry(&entry).expect("second");
        assert_eq!(k1, k2);
    }

    #[test]
    fn two_distinct_entries_generate_different_keys() {
        let entry_a = mock_entry();
        let entry_b = mock_entry();
        let ka = get_or_create_with_entry(&entry_a).expect("a");
        let kb = get_or_create_with_entry(&entry_b).expect("b");
        assert_ne!(
            ka, kb,
            "CSPRNG should produce distinct keys per fresh entry"
        );
    }
}

#[cfg(test)]
mod decode_tests {
    use super::*;

    #[test]
    fn decode_key_rejects_wrong_length() {
        let short = STANDARD.encode(&[0u8; 16][..]);
        assert!(decode_key(&short).is_err());
        let long = STANDARD.encode(&[0u8; 64][..]);
        assert!(decode_key(&long).is_err());
    }

    #[test]
    fn decode_key_accepts_32_bytes() {
        let exact = STANDARD.encode(&[0xAAu8; 32][..]);
        assert!(decode_key(&exact).is_ok());
    }

    #[test]
    fn decode_key_rejects_invalid_base64() {
        assert!(decode_key("not!valid!base64!").is_err());
    }
}

#[cfg(test)]
mod service_tests {
    use super::service_for;

    #[test]
    fn production_identifier_keeps_the_historic_service() {
        assert_eq!(service_for("com.synk.health"), "synk");
    }

    #[test]
    fn any_other_identifier_gets_the_dev_service() {
        assert_eq!(service_for("com.synk.health.dev"), "synk-dev");
        assert_eq!(service_for(""), "synk-dev");
    }
}
