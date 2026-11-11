use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};

pub fn record_id(
    prefix: &str,
    kind: &str,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    value: &str,
    origin: &str,
) -> String {
    let mut hasher = Sha256::new();
    for part in [
        kind,
        &start.timestamp().to_string(),
        &end.timestamp().to_string(),
        value,
        origin,
    ] {
        hasher.update((part.len() as u64).to_le_bytes());
        hasher.update(part.as_bytes());
    }
    let digest = hasher.finalize();
    let hex: String = digest.iter().take(16).map(|b| format!("{b:02x}")).collect();
    format!("{prefix}:{hex}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at(h: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 25, h, 0, 0).unwrap()
    }

    #[test]
    fn the_fingerprint_is_stable_and_prefixed() {
        let a = record_id("ah", "BodyMass", at(7), at(7), "72.4", "Withings");
        let b = record_id("ah", "BodyMass", at(7), at(7), "72.4", "Withings");
        assert_eq!(a, b);
        assert!(a.starts_with("ah:"), "{a}");
        assert_eq!(a.len(), 3 + 32);
        let other = record_id("fit", "BodyMass", at(7), at(7), "72.4", "Withings");
        assert!(other.starts_with("fit:"), "{other}");
        assert_eq!(a.strip_prefix("ah:"), other.strip_prefix("fit:"));
    }

    #[test]
    fn every_ingredient_changes_the_fingerprint() {
        let base = record_id("ah", "BodyMass", at(7), at(7), "72.4", "Withings");
        assert_ne!(
            base,
            record_id("ah", "Height", at(7), at(7), "72.4", "Withings")
        );
        assert_ne!(
            base,
            record_id("ah", "BodyMass", at(8), at(7), "72.4", "Withings")
        );
        assert_ne!(
            base,
            record_id("ah", "BodyMass", at(7), at(8), "72.4", "Withings")
        );
        assert_ne!(
            base,
            record_id("ah", "BodyMass", at(7), at(7), "72.5", "Withings")
        );
        assert_ne!(
            base,
            record_id("ah", "BodyMass", at(7), at(7), "72.4", "Oura")
        );
    }

    #[test]
    fn separators_cannot_be_forged_by_the_fields() {
        let a = record_id("ah", "A|B", at(7), at(7), "1", "x");
        let b = record_id("ah", "A", at(7), at(7), "B|1", "x");
        assert_ne!(a, b);
    }
}
