pub(crate) mod cipher;
pub(crate) mod keyring;
pub(crate) mod migration;
pub(crate) mod recovery;

pub use migration::{migrate_plaintext_secrets, MigrationReport};
pub use recovery::{invalidate_undecryptable_secrets, InvalidatedSecrets, InvalidatedSource};

use zeroize::{Zeroize, Zeroizing};

use crate::error::AppError;

pub struct CryptoVault {
    key: [u8; 32],
}

impl CryptoVault {
    pub fn init(service: &str) -> Result<Self, AppError> {
        Ok(Self {
            key: *keyring::get_or_create_master_key(service)?,
        })
    }

    pub fn from_key(key: [u8; 32]) -> Self {
        Self { key }
    }

    pub fn seal_str(&self, plaintext: &str) -> Result<String, AppError> {
        cipher::seal(plaintext, &self.key)
    }

    pub fn open_str(&self, value: &str) -> Result<Zeroizing<String>, AppError> {
        if value.is_empty() {
            return Ok(Zeroizing::new(String::new()));
        }
        if !cipher::is_encrypted(value) {
            return Err(AppError::Crypto(
                "valeur attendue chiffree (enc:v1:), trouvee en clair".to_string(),
            ));
        }
        cipher::open(value, &self.key)
    }

    pub fn is_encrypted(value: &str) -> bool {
        cipher::is_encrypted(value)
    }

    fn wipe(&mut self) {
        self.key.zeroize();
    }
}

impl Drop for CryptoVault {
    fn drop(&mut self) {
        self.wipe();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vault() -> CryptoVault {
        CryptoVault::from_key([0x42u8; 32])
    }

    #[test]
    fn seal_str_then_open_str_round_trip() {
        let v = vault();
        let sealed = v.seal_str("hello").unwrap();
        assert!(sealed.starts_with("enc:v1:"));
        let opened = v.open_str(&sealed).unwrap();
        assert_eq!(opened.as_str(), "hello");
    }

    #[test]
    fn an_opened_secret_is_wiped_when_dropped() {
        let v = vault();
        let sealed = v.seal_str("jeton").unwrap();
        let opened: Zeroizing<String> = v.open_str(&sealed).unwrap();
        assert_eq!(opened.as_str(), "jeton");
    }

    #[test]
    fn open_str_refuses_plaintext() {
        assert!(vault().open_str("not_encrypted_value").is_err());
    }

    #[test]
    fn open_str_keeps_empty_as_absent() {
        assert_eq!(vault().open_str("").unwrap().as_str(), "");
    }

    #[test]
    fn wipe_zeroes_the_key() {
        let mut v = vault();
        v.wipe();
        assert_eq!(v.key, [0u8; 32]);
    }

    #[test]
    fn open_str_rejects_corrupted_encrypted() {
        let v = vault();
        assert!(v.open_str("enc:v1:invalid_b64!!").is_err());
    }

    #[test]
    fn static_is_encrypted_detects_prefix() {
        assert!(CryptoVault::is_encrypted("enc:v1:xyz"));
        assert!(!CryptoVault::is_encrypted("plain"));
    }
}
