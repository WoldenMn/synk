/// Traduit un model_id Withings en nom commercial. `None` si inconnu.
pub fn friendly_device_name(model_id: i64) -> Option<String> {
    let name = match model_id {
        4 => "Smart Body Analyzer",
        5 => "Body+",
        6 => "Body Cardio",
        7 => "Body",
        9 => "Body Pro",
        10 => "Body Scan",
        41 => "Tensiometre iOS",
        42 => "Tensiometre sans fil",
        43 => "BPM",
        44 => "BPM Core",
        45 => "BPM Connect",
        46 => "BPM Connect Pro",
        51 => "Pulse Ox",
        52 => "Activite",
        53 => "Activite Pop / Steel",
        54 => "Go",
        55 => "Steel HR",
        58 => "Pulse HR",
        59 => "Steel HR Sport",
        60 => "Aura Dock",
        61 => "Sleep Analyzer",
        62 => "Sleep Analyzer V2",
        63 => "Sleep Analyzer",
        70 => "Thermo",
        90 => "Move",
        91 => "Move ECG",
        93 => "ScanWatch",
        _ => return None,
    };
    Some(name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_models_resolve() {
        assert_eq!(friendly_device_name(10).as_deref(), Some("Body Scan"));
        assert_eq!(friendly_device_name(93).as_deref(), Some("ScanWatch"));
    }

    #[test]
    fn unknown_model_is_none() {
        assert_eq!(friendly_device_name(0), None);
        assert_eq!(friendly_device_name(9999), None);
    }
}
