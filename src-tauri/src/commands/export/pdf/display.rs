// Mise en forme francaise des valeurs ecrites dans les PDF : virgule decimale,
// dates jj/mm/aaaa, durees en heures et minutes. Les PDF utilisent les polices
// integrees (encodage WinAnsi) : accents et « Â » passent, pas les espaces fines.

/// Nombre a `decimals` decimales, virgule francaise : 76.037 -> « 76,0 ».
pub(crate) fn fr_number(value: f64, decimals: usize) -> String {
    format!("{value:.decimals$}").replace('.', ",")
}

/// Valeur suivie de son unite ; une decimale pour les masses, pourcentages et
/// longueurs, aucune pour les comptes (pas, bpm, kcal). Absente : « - ».
pub(crate) fn fr_measure(value: Option<f64>, unit: &str) -> String {
    match value {
        Some(v) => {
            let decimals = if matches!(unit, "kg" | "%" | "cm" | "km") {
                1
            } else {
                0
            };
            format!("{} {unit}", fr_number(v, decimals))
        }
        None => "-".to_string(),
    }
}

/// Duree en secondes : 41640 -> « 11 h 34 », 2040 -> « 34 min ».
pub(crate) fn fr_duration(seconds: f64) -> String {
    let total_min = (seconds.max(0.0) / 60.0).round() as i64;
    let (h, m) = (total_min / 60, total_min % 60);
    if h == 0 {
        format!("{m} min")
    } else {
        format!("{h} h {m:02}")
    }
}

/// Date ISO (jour seul ou horodatage) en jj/mm/aaaa ; une chaine qui n'est pas
/// une date est rendue telle quelle plutot que masquee.
pub(crate) fn fr_date(iso: &str) -> String {
    let day = iso.get(..10).unwrap_or(iso);
    match chrono::NaiveDate::parse_from_str(day, "%Y-%m-%d") {
        Ok(d) => d.format("%d/%m/%Y").to_string(),
        Err(_) => iso.to_string(),
    }
}

/// Valeur d'un record de la carte d'identite (unite « s » : duree en secondes).
pub(crate) fn record_value(value: f64, unit: &str) -> String {
    match unit {
        "s" => fr_duration(value),
        "/100" => format!("{}/100", fr_number(value, 0)),
        _ => fr_measure(Some(value), unit),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounds_with_a_french_decimal_comma() {
        assert_eq!(fr_number(21.647000000000002, 1), "21,6");
        assert_eq!(fr_measure(Some(76.037), "kg"), "76,0 kg");
        assert_eq!(fr_measure(Some(12345.0), "pas"), "12345 pas");
        assert_eq!(fr_measure(None, "kg"), "-");
    }

    #[test]
    fn renders_durations_in_hours_and_minutes() {
        assert_eq!(fr_duration(41640.0), "11 h 34");
        assert_eq!(fr_duration(2040.0), "34 min");
        assert_eq!(fr_duration(3600.0), "1 h 00");
    }

    #[test]
    fn renders_dates_day_first_and_keeps_what_is_not_a_date() {
        assert_eq!(fr_date("2026-06-28T06:53:52Z"), "28/06/2026");
        assert_eq!(fr_date("2026-04-07"), "07/04/2026");
        assert_eq!(fr_date("hier"), "hier");
    }

    #[test]
    fn formats_records_by_unit() {
        assert_eq!(record_value(41640.0, "s"), "11 h 34");
        assert_eq!(record_value(93.0, "/100"), "93/100");
        assert_eq!(record_value(43.0, "bpm"), "43 bpm");
        assert_eq!(record_value(70.287, "kg"), "70,3 kg");
    }
}
