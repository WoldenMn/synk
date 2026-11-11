use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit};
use base64::{engine::general_purpose::STANDARD, Engine as _};

use crate::error::AppError;
use zeroize::Zeroizing;

pub(crate) const PREFIX: &str = "enc:v1:";
const NONCE_LEN: usize = 12;
const TAG_LEN: usize = 16;

pub(crate) fn seal(plaintext: &str, key: &[u8; 32]) -> Result<String, AppError> {
    let cipher = Aes256Gcm::new(key.into());
    let mut nonce = [0u8; NONCE_LEN];
    getrandom::fill(&mut nonce).map_err(|e| AppError::Crypto(format!("seal rng: {e}")))?;

    let ct = cipher
        .encrypt((&nonce).into(), plaintext.as_bytes())
        .map_err(|e| AppError::Crypto(format!("seal: {e}")))?;

    let mut blob = Vec::with_capacity(NONCE_LEN + ct.len());
    blob.extend_from_slice(&nonce);
    blob.extend_from_slice(&ct);
    Ok(format!("{}{}", PREFIX, STANDARD.encode(&blob)))
}

pub(crate) fn open(prefixed: &str, key: &[u8; 32]) -> Result<Zeroizing<String>, AppError> {
    let b64 = prefixed
        .strip_prefix(PREFIX)
        .ok_or_else(|| AppError::Crypto("missing enc:v1: prefix".to_string()))?;
    let blob = Zeroizing::new(
        STANDARD
            .decode(b64)
            .map_err(|e| AppError::Crypto(format!("open base64: {e}")))?,
    );
    if blob.len() < NONCE_LEN + TAG_LEN {
        return Err(AppError::Crypto("ciphertext too short".to_string()));
    }
    let (nonce, ct) = blob
        .split_first_chunk::<NONCE_LEN>()
        .ok_or_else(|| AppError::Crypto("ciphertext too short".to_string()))?;
    let cipher = Aes256Gcm::new(key.into());
    let pt = Zeroizing::new(
        cipher
            .decrypt(nonce.into(), ct)
            .map_err(|e| AppError::Crypto(format!("open: {e}")))?,
    );
    std::str::from_utf8(&pt)
        .map(|text| Zeroizing::new(text.to_string()))
        .map_err(|e| AppError::Crypto(format!("open utf8: {e}")))
}

pub(crate) fn is_encrypted(value: &str) -> bool {
    value.starts_with(PREFIX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key32() -> [u8; 32] {
        [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
            0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c,
            0x1d, 0x1e, 0x1f, 0x20,
        ]
    }

    #[test]
    fn seal_open_round_trip() {
        let key = key32();
        let plaintext = "hello secret world";
        let sealed = seal(plaintext, &key).expect("seal");
        assert!(sealed.starts_with(PREFIX), "missing prefix: {sealed}");
        let opened = open(&sealed, &key).expect("open");
        assert_eq!(opened.as_str(), plaintext);
    }

    #[test]
    fn seal_produces_different_ciphertexts_for_same_input() {
        let key = key32();
        let s1 = seal("same", &key).unwrap();
        let s2 = seal("same", &key).unwrap();
        assert_ne!(s1, s2);
    }

    #[test]
    fn open_rejects_tampered_ciphertext() {
        let key = key32();
        let sealed = seal("secret", &key).unwrap();
        let mut chars: Vec<char> = sealed.chars().collect();
        let mid = chars.len() / 2;
        chars[mid] = if chars[mid] == 'A' { 'B' } else { 'A' };
        let tampered: String = chars.into_iter().collect();
        assert!(open(&tampered, &key).is_err());
    }

    #[test]
    fn open_rejects_wrong_key() {
        let k1 = key32();
        let mut k2 = key32();
        k2[0] ^= 0xff;
        let sealed = seal("secret", &k1).unwrap();
        assert!(open(&sealed, &k2).is_err());
    }

    #[test]
    fn is_encrypted_detects_prefix() {
        assert!(is_encrypted("enc:v1:somebase64"));
        assert!(!is_encrypted("plaintext"));
        assert!(!is_encrypted(""));
        assert!(!is_encrypted("enc:v2:future"));
    }

    #[test]
    fn open_rejects_short_ciphertext() {
        let key = key32();
        assert!(open("enc:v1:AAAA", &key).is_err());
    }

    #[test]
    fn open_rejects_missing_prefix() {
        let key = key32();
        assert!(open("notencrypted", &key).is_err());
    }

    const GOLDEN: &str = "enc:v1:AAECAwQFBgcICQoLGVVt+6f9Em5sWdc1RO+Y/0Xz6y1qr7kQSnej1TwRe2UKKw==";

    #[test]
    fn opens_the_reference_vector() {
        assert_eq!(
            open(GOLDEN, &key32()).unwrap().as_str(),
            "hello secret world"
        );
    }

    #[test]
    fn refuses_the_reference_vector_under_another_key() {
        let mut key = key32();
        key[0] ^= 0xff;
        assert!(open(GOLDEN, &key).is_err());
    }
}
