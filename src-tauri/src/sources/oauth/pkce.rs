use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::error::AppError;

pub struct Verifier(Zeroizing<String>);

impl Verifier {
    pub fn generate() -> Result<Verifier, AppError> {
        let mut bytes = Zeroizing::new([0u8; 64]);
        getrandom::fill(bytes.as_mut_slice())
            .map_err(|e| AppError::Crypto(format!("pkce rng: {e}")))?;
        Ok(Verifier(Zeroizing::new(URL_SAFE_NO_PAD.encode(*bytes))))
    }

    pub fn from_secret(value: &str) -> Verifier {
        Verifier(Zeroizing::new(value.to_string()))
    }

    pub fn expose(&self) -> &str {
        &self.0
    }

    pub fn challenge(&self) -> String {
        URL_SAFE_NO_PAD.encode(Sha256::digest(self.0.as_bytes()))
    }
}

impl std::fmt::Debug for Verifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Verifier([masque])")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RFC7636_VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
    const RFC7636_CHALLENGE: &str = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";

    #[test]
    fn the_challenge_matches_the_rfc_7636_appendix_b_vector() {
        let v = Verifier::from_secret(RFC7636_VERIFIER);
        assert_eq!(v.challenge(), RFC7636_CHALLENGE);
    }

    #[test]
    fn a_verifier_is_86_unreserved_characters_from_64_random_bytes() {
        let v = Verifier::generate().expect("entropie");
        assert_eq!(v.expose().len(), 86);
        assert!(
            v.expose()
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'),
            "caracteres hors du jeu non reserve"
        );
    }

    #[test]
    fn two_verifiers_differ() {
        let a = Verifier::generate().expect("entropie");
        let b = Verifier::generate().expect("entropie");
        assert_ne!(a.expose(), b.expose());
    }

    #[test]
    fn the_debug_output_hides_the_verifier() {
        let v = Verifier::from_secret("SECRET-VERIFIER-VALUE");
        let shown = format!("{v:?}");
        assert!(!shown.contains("SECRET"), "{shown}");
    }
}
