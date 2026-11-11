use super::measures::{
    MEASURE_BMR, MEASURE_BODY_TEMPERATURE, MEASURE_EXTRACELLULAR_WATER,
    MEASURE_INTRACELLULAR_WATER, MEASURE_NERVE_HEALTH, MEASURE_PULSE_WAVE_VELOCITY,
    MEASURE_SKIN_TEMPERATURE, MEASURE_SPO2, MEASURE_TEMPERATURE, MEASURE_VASCULAR_AGE,
    MEASURE_VISCERAL_FAT, MEASURE_VO2MAX,
};

pub fn vital_type_name(measure_type: i64) -> Option<&'static str> {
    match measure_type {
        MEASURE_TEMPERATURE => Some("temperature"),
        MEASURE_SPO2 => Some("spo2"),
        MEASURE_BODY_TEMPERATURE => Some("body_temperature"),
        MEASURE_SKIN_TEMPERATURE => Some("skin_temperature"),
        MEASURE_PULSE_WAVE_VELOCITY => Some("pulse_wave_velocity"),
        MEASURE_VO2MAX => Some("vo2max"),
        MEASURE_VASCULAR_AGE => Some("vascular_age"),
        MEASURE_NERVE_HEALTH => Some("nerve_health"),
        MEASURE_VISCERAL_FAT => Some("visceral_fat"),
        MEASURE_EXTRACELLULAR_WATER => Some("extracellular_water"),
        MEASURE_INTRACELLULAR_WATER => Some("intracellular_water"),
        MEASURE_BMR => Some("bmr"),
        _ => None,
    }
}

#[cfg(test)]
mod helpers_tests {
    use super::*;

    #[test]
    fn vital_type_name_known_types() {
        assert_eq!(vital_type_name(MEASURE_TEMPERATURE), Some("temperature"));
        assert_eq!(vital_type_name(MEASURE_SPO2), Some("spo2"));
        assert_eq!(vital_type_name(MEASURE_VO2MAX), Some("vo2max"));
    }

    #[test]
    fn vital_type_name_unknown() {
        assert_eq!(vital_type_name(1), None);
        assert_eq!(vital_type_name(9999), None);
    }
}
