/// Convertit un timestamp Unix en ISO-8601 UTC.
/// Retourne `None` si le timestamp est hors de la plage valide de chrono
/// (évite le stockage de chaîne vide en DB).
pub fn ts_to_iso(ts: i64) -> Option<String> {
    chrono::DateTime::from_timestamp(ts, 0).map(|d| d.format("%Y-%m-%dT%H:%M:%SZ").to_string())
}

/// Convertit un timestamp Unix optionnel en ISO-8601 UTC (`Option<String>`).
/// Retourne `None` si le timestamp est hors de la plage valide de chrono.
pub fn opt_ts_to_iso(ts: Option<i64>) -> Option<String> {
    ts.and_then(|t| {
        chrono::DateTime::from_timestamp(t, 0).map(|d| d.format("%Y-%m-%dT%H:%M:%SZ").to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ts_to_iso_valid_timestamp() {
        // 2023-11-15 00:00:00 UTC = 1700006400
        let result = ts_to_iso(1_700_006_400);
        assert_eq!(result, Some("2023-11-15T00:00:00Z".to_string()));
    }

    #[test]
    fn ts_to_iso_out_of_range_returns_none() {
        // i64::MAX est hors plage valide de chrono
        let result = ts_to_iso(i64::MAX);
        assert!(result.is_none(), "timestamp hors plage doit retourner None");
    }

    #[test]
    fn opt_ts_to_iso_some_valid() {
        let result = opt_ts_to_iso(Some(1_700_006_400));
        assert_eq!(result, Some("2023-11-15T00:00:00Z".to_string()));
    }

    #[test]
    fn opt_ts_to_iso_none_input() {
        assert_eq!(opt_ts_to_iso(None), None);
    }

    #[test]
    fn opt_ts_to_iso_out_of_range_returns_none() {
        assert!(opt_ts_to_iso(Some(i64::MAX)).is_none());
    }
}
