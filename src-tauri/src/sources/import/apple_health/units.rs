const LB_KG: f64 = 0.453_592_37;
const ST_KG: f64 = 6.350_293_18;
const MILE_M: f64 = 1609.344;
const FOOT_M: f64 = 0.3048;
const YARD_M: f64 = 0.9144;
const KCAL_KJ: f64 = 4.184;
const GLUCOSE_MG_DL_PER_MMOL_L: f64 = 18.015_58;

pub fn mass_kg(value: f64, unit: &str) -> Option<f64> {
    match unit {
        "kg" => Some(value),
        "g" => Some(value / 1000.0),
        "lb" => Some(value * LB_KG),
        "oz" => Some(value * LB_KG / 16.0),
        "st" => Some(value * ST_KG),
        _ => None,
    }
}

pub fn distance_m(value: f64, unit: &str) -> Option<f64> {
    match unit {
        "m" => Some(value),
        "km" => Some(value * 1000.0),
        "mi" => Some(value * MILE_M),
        "ft" => Some(value * FOOT_M),
        "yd" => Some(value * YARD_M),
        _ => None,
    }
}

pub fn energy_kcal(value: f64, unit: &str) -> Option<f64> {
    match unit {
        "kcal" | "Cal" => Some(value),
        "cal" => Some(value / 1000.0),
        "kJ" => Some(value / KCAL_KJ),
        "J" => Some(value / 1000.0 / KCAL_KJ),
        _ => None,
    }
}

pub fn temperature_c(value: f64, unit: &str) -> Option<f64> {
    match unit {
        "degC" => Some(value),
        "degF" => Some((value - 32.0) * 5.0 / 9.0),
        "K" => Some(value - 273.15),
        _ => None,
    }
}

pub fn glucose_mmol_l(value: f64, unit: &str) -> Option<f64> {
    if unit.starts_with("mmol") && unit.ends_with("/L") {
        return Some(value);
    }
    match unit {
        "mg/dL" => Some(value / GLUCOSE_MG_DL_PER_MMOL_L),
        _ => None,
    }
}

pub fn duration_s(value: f64, unit: &str) -> Option<f64> {
    match unit {
        "s" => Some(value),
        "min" => Some(value * 60.0),
        "hr" => Some(value * 3600.0),
        _ => None,
    }
}

pub fn percent(value: f64, unit: &str) -> Option<f64> {
    if unit == "%" && (0.0..=1.0).contains(&value) {
        Some(value * 100.0)
    } else {
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Option<f64>, b: f64) {
        let a = a.expect("conversion");
        assert!((a - b).abs() < 1e-6, "{a} != {b}");
    }

    #[test]
    fn masses_come_back_in_kilograms() {
        close(mass_kg(72.4, "kg"), 72.4);
        close(mass_kg(160.0, "lb"), 72.574_779_2);
        close(mass_kg(72_400.0, "g"), 72.4);
        close(mass_kg(11.0, "st"), 69.853_225);
        assert_eq!(mass_kg(1.0, "furlong"), None);
    }

    #[test]
    fn distances_come_back_in_metres() {
        close(distance_m(5.2, "km"), 5200.0);
        close(distance_m(3.0, "mi"), 4828.032);
        close(distance_m(800.0, "m"), 800.0);
        close(distance_m(10.0, "ft"), 3.048);
        assert_eq!(distance_m(1.0, "kg"), None);
    }

    #[test]
    fn energies_come_back_in_kilocalories() {
        close(energy_kcal(320.0, "kcal"), 320.0);
        close(energy_kcal(320.0, "Cal"), 320.0);
        close(energy_kcal(1000.0, "cal"), 1.0);
        close(energy_kcal(418.4, "kJ"), 100.0);
        assert_eq!(energy_kcal(1.0, "J/s"), None);
    }

    #[test]
    fn temperatures_come_back_in_celsius() {
        close(temperature_c(36.6, "degC"), 36.6);
        close(temperature_c(98.6, "degF"), 37.0);
        close(temperature_c(310.15, "K"), 37.0);
        assert_eq!(temperature_c(1.0, "rankine"), None);
    }

    #[test]
    fn glucose_comes_back_in_mmol_per_litre_whatever_the_unit_spelling() {
        close(glucose_mmol_l(5.5, "mmol<180.1558800000541>/L"), 5.5);
        close(glucose_mmol_l(5.5, "mmol/L"), 5.5);
        close(glucose_mmol_l(99.0, "mg/dL"), 5.495_243_561);
        assert_eq!(glucose_mmol_l(1.0, "g/L"), None);
    }

    #[test]
    fn durations_come_back_in_seconds() {
        close(duration_s(30.5, "min"), 1830.0);
        close(duration_s(90.0, "s"), 90.0);
        close(duration_s(1.5, "hr"), 5400.0);
        assert_eq!(duration_s(1.0, "parsec"), None);
    }

    #[test]
    fn a_fraction_stored_with_a_percent_unit_is_scaled_to_a_percentage() {
        close(percent(0.186, "%"), 18.6);
        close(percent(0.97, "%"), 97.0);
        close(percent(97.0, "%"), 97.0);
        close(percent(18.6, ""), 18.6);
    }
}
