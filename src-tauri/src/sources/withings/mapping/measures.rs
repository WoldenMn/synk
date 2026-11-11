pub const MEASURE_WEIGHT: i64 = 1;
pub const MEASURE_FAT_FREE_MASS: i64 = 5;
pub const MEASURE_FAT_RATIO: i64 = 6;
pub const MEASURE_FAT_MASS: i64 = 8;
pub const MEASURE_DIASTOLIC: i64 = 9;
pub const MEASURE_SYSTOLIC: i64 = 10;
pub const MEASURE_HEART_RATE: i64 = 11;
pub const MEASURE_TEMPERATURE: i64 = 12;
pub const MEASURE_SPO2: i64 = 54;
pub const MEASURE_BODY_TEMPERATURE: i64 = 71;
pub const MEASURE_SKIN_TEMPERATURE: i64 = 73;
pub const MEASURE_MUSCLE_MASS: i64 = 76;
pub const MEASURE_HYDRATION: i64 = 77;
pub const MEASURE_BONE_MASS: i64 = 88;
pub const MEASURE_PULSE_WAVE_VELOCITY: i64 = 91;
pub const MEASURE_VO2MAX: i64 = 123;
pub const MEASURE_VASCULAR_AGE: i64 = 155;
pub const MEASURE_NERVE_HEALTH: i64 = 167;
pub const MEASURE_EXTRACELLULAR_WATER: i64 = 168;
pub const MEASURE_INTRACELLULAR_WATER: i64 = 169;
pub const MEASURE_VISCERAL_FAT: i64 = 170;
pub const MEASURE_BMR: i64 = 226;

/// Measure types par categorie de fetch /measure (source unique de verite,
/// remplace les strings brutes de sync/measures.rs).
pub const BODY_TYPES: &[i64] = &[
    MEASURE_WEIGHT,
    MEASURE_FAT_FREE_MASS,
    MEASURE_FAT_RATIO,
    MEASURE_FAT_MASS,
    MEASURE_MUSCLE_MASS,
    MEASURE_HYDRATION,
    MEASURE_BONE_MASS,
];
pub const BP_TYPES: &[i64] = &[MEASURE_DIASTOLIC, MEASURE_SYSTOLIC, MEASURE_HEART_RATE];
pub const VITAL_TYPES: &[i64] = &[
    MEASURE_TEMPERATURE,
    MEASURE_SPO2,
    MEASURE_BODY_TEMPERATURE,
    MEASURE_SKIN_TEMPERATURE,
    MEASURE_PULSE_WAVE_VELOCITY,
    MEASURE_VO2MAX,
    MEASURE_VASCULAR_AGE,
    MEASURE_NERVE_HEALTH,
    MEASURE_EXTRACELLULAR_WATER,
    MEASURE_INTRACELLULAR_WATER,
    MEASURE_VISCERAL_FAT,
    MEASURE_BMR,
];

/// Joint des measure types en chaine CSV pour le parametre `meastypes` de /measure.
pub fn join_meastypes(types: &[i64]) -> String {
    types
        .iter()
        .map(|t| t.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

pub fn convert_measure(value: i64, unit: i64) -> f64 {
    value as f64 * 10_f64.powi(unit as i32)
}

/// Rejette les valeurs non stockables (NaN, +/-Infinity) et hors des bornes
/// optionnelles de la definition metrique. None => la mesure doit etre ignoree.
/// Aligne les chemins body/vital/metrics sur le garde-fou de upsert_bp_group.
pub fn sanitize_measure(value: f64, min: Option<f64>, max: Option<f64>) -> Option<f64> {
    if !value.is_finite() {
        return None;
    }
    if let Some(lo) = min {
        if value < lo {
            return None;
        }
    }
    if let Some(hi) = max {
        if value > hi {
            return None;
        }
    }
    Some(value)
}

pub fn is_body_composition(measure_type: i64) -> bool {
    matches!(
        measure_type,
        MEASURE_WEIGHT
            | MEASURE_FAT_FREE_MASS
            | MEASURE_FAT_RATIO
            | MEASURE_FAT_MASS
            | MEASURE_MUSCLE_MASS
            | MEASURE_HYDRATION
            | MEASURE_BONE_MASS
    )
}

pub fn is_blood_pressure(measure_type: i64) -> bool {
    matches!(
        measure_type,
        MEASURE_DIASTOLIC | MEASURE_SYSTOLIC | MEASURE_HEART_RATE
    )
}

/// Resout un Withings measure type en slug metric_definitions correspondant.
/// Retourne None si le type n'est pas encore mappe (ex: types pression, heart).
pub fn resolve_metric_slug(withings_type: i64) -> Option<&'static str> {
    match withings_type {
        MEASURE_WEIGHT => Some("weight"),
        MEASURE_FAT_FREE_MASS => Some("fat_free_mass"),
        MEASURE_FAT_RATIO => Some("fat_ratio"),
        MEASURE_FAT_MASS => Some("fat_mass"),
        // V2.7 : BP slugs sont desormais integres au stack metriques V2 (categorie vital_bp)
        MEASURE_DIASTOLIC => Some("diastolic"),
        MEASURE_SYSTOLIC => Some("systolic"),
        MEASURE_HEART_RATE => Some("heart_rate"),
        MEASURE_TEMPERATURE => Some("temperature"),
        MEASURE_SPO2 => Some("spo2"),
        MEASURE_BODY_TEMPERATURE => Some("body_temperature"),
        MEASURE_SKIN_TEMPERATURE => Some("skin_temperature"),
        MEASURE_MUSCLE_MASS => Some("muscle_mass"),
        MEASURE_HYDRATION => Some("hydration"),
        MEASURE_BONE_MASS => Some("bone_mass"),
        MEASURE_PULSE_WAVE_VELOCITY => Some("pulse_wave_velocity"),
        MEASURE_VO2MAX => Some("vo2max"),
        MEASURE_VASCULAR_AGE => Some("vascular_age"),
        MEASURE_NERVE_HEALTH => Some("nerve_health"),
        MEASURE_EXTRACELLULAR_WATER => Some("extracellular_water"),
        MEASURE_INTRACELLULAR_WATER => Some("intracellular_water"),
        MEASURE_VISCERAL_FAT => Some("visceral_fat"),
        MEASURE_BMR => Some("bmr"),
        _ => None,
    }
}

#[cfg(test)]
mod resolve_tests {
    use super::*;

    #[test]
    fn test_resolve_known_types() {
        assert_eq!(resolve_metric_slug(1), Some("weight"));
        assert_eq!(resolve_metric_slug(6), Some("fat_ratio"));
        assert_eq!(resolve_metric_slug(76), Some("muscle_mass"));
        assert_eq!(resolve_metric_slug(123), Some("vo2max"));
        assert_eq!(resolve_metric_slug(226), Some("bmr"));
        // V2.7 : BP slugs sont desormais dans le stack metriques V2
        assert_eq!(resolve_metric_slug(9), Some("diastolic"));
        assert_eq!(resolve_metric_slug(10), Some("systolic"));
        assert_eq!(resolve_metric_slug(11), Some("heart_rate"));
        assert_eq!(resolve_metric_slug(12), Some("temperature"));
    }

    #[test]
    fn test_resolve_unknown_types() {
        assert_eq!(resolve_metric_slug(0), None);
        assert_eq!(resolve_metric_slug(999), None);
    }
}

#[cfg(test)]
mod helpers_tests {
    use super::*;

    #[test]
    fn convert_measure_applies_positive_exponent() {
        assert_eq!(convert_measure(123, 0), 123.0);
        assert_eq!(convert_measure(5, 2), 500.0);
    }

    #[test]
    fn convert_measure_applies_negative_exponent() {
        assert!((convert_measure(725, -1) - 72.5).abs() < 0.001);
        assert!((convert_measure(12345, -2) - 123.45).abs() < 0.001);
    }

    #[test]
    fn is_body_composition_positive() {
        assert!(is_body_composition(MEASURE_WEIGHT));
        assert!(is_body_composition(MEASURE_FAT_RATIO));
        assert!(is_body_composition(MEASURE_HYDRATION));
    }

    #[test]
    fn is_body_composition_negative() {
        assert!(!is_body_composition(MEASURE_SYSTOLIC));
        assert!(!is_body_composition(MEASURE_SPO2));
    }

    #[test]
    fn is_blood_pressure_positive() {
        assert!(is_blood_pressure(MEASURE_SYSTOLIC));
        assert!(is_blood_pressure(MEASURE_DIASTOLIC));
        assert!(is_blood_pressure(MEASURE_HEART_RATE));
    }

    #[test]
    fn is_blood_pressure_negative() {
        assert!(!is_blood_pressure(MEASURE_WEIGHT));
    }

    #[test]
    fn meastype_slices_are_consistent() {
        for &t in BODY_TYPES {
            assert!(is_body_composition(t), "BODY_TYPES contient {t} non body");
        }
        for &t in BP_TYPES {
            assert!(is_blood_pressure(t), "BP_TYPES contient {t} non bp");
        }
        for &t in VITAL_TYPES {
            assert!(
                crate::sources::withings::mapping::vital_type_name(t).is_some(),
                "VITAL_TYPES contient {t} non vital"
            );
        }
    }
}

#[cfg(test)]
mod sanitize_tests {
    use super::*;

    #[test]
    fn accepts_finite_without_bounds() {
        assert_eq!(sanitize_measure(72.5, None, None), Some(72.5));
        assert_eq!(sanitize_measure(0.0, None, None), Some(0.0));
        assert_eq!(sanitize_measure(-5.0, None, None), Some(-5.0));
    }

    #[test]
    fn rejects_nan_and_infinity() {
        assert_eq!(sanitize_measure(f64::NAN, None, None), None);
        assert_eq!(sanitize_measure(f64::INFINITY, None, None), None);
        assert_eq!(sanitize_measure(f64::NEG_INFINITY, None, None), None);
        // Cas reels du bug #21 : convert_measure(0,400)=0*inf=NaN ; convert_measure(1,309)=inf
        assert_eq!(sanitize_measure(convert_measure(0, 400), None, None), None);
        assert_eq!(sanitize_measure(convert_measure(1, 309), None, None), None);
    }

    #[test]
    fn enforces_min_max_bounds() {
        assert_eq!(sanitize_measure(10.0, Some(20.0), Some(300.0)), None);
        assert_eq!(sanitize_measure(350.0, Some(20.0), Some(300.0)), None);
        assert_eq!(sanitize_measure(75.0, Some(20.0), Some(300.0)), Some(75.0));
        assert_eq!(sanitize_measure(20.0, Some(20.0), Some(300.0)), Some(20.0));
        assert_eq!(
            sanitize_measure(300.0, Some(20.0), Some(300.0)),
            Some(300.0)
        );
    }
}
