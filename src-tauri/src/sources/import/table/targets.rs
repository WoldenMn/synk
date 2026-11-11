use serde::Serialize;

use crate::sources::import::apple_health::units::{glucose_mmol_l, mass_kg, temperature_c};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Observation,
    Systolic,
    Diastolic,
}

#[derive(Debug, Clone, Copy)]
pub struct Unit {
    pub id: &'static str,
    pub label: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct Target {
    pub metric: &'static str,
    pub label: &'static str,
    pub kind: Kind,
    pub stored_unit: &'static str,
    pub units: &'static [Unit],
    pub keywords: &'static [&'static str],
}

const fn unit(id: &'static str, label: &'static str) -> Unit {
    Unit { id, label }
}

const MASS: &[Unit] = &[
    unit("kg", "kg"),
    unit("lb", "lb"),
    unit("st", "stone"),
    unit("g", "g"),
];
const PERCENT: &[Unit] = &[unit("%", "%"), unit("fraction", "fraction (0 à 1)")];

pub static TARGETS: &[Target] = &[
    Target {
        metric: "weight",
        label: "Poids",
        kind: Kind::Observation,
        stored_unit: "kg",
        units: MASS,
        keywords: &["weight", "poids", "gewicht", "peso"],
    },
    Target {
        metric: "fat_mass",
        label: "Masse grasse (poids)",
        kind: Kind::Observation,
        stored_unit: "kg",
        units: MASS,
        keywords: &["fat mass", "masse grasse (kg"],
    },
    Target {
        metric: "fat_ratio",
        label: "Masse grasse (%)",
        kind: Kind::Observation,
        stored_unit: "%",
        units: PERCENT,
        keywords: &[
            "body fat", "fat(", "graisse", "grasse", "fat %", "fat%", "bodyfat",
        ],
    },
    Target {
        metric: "fat_free_mass",
        label: "Masse maigre",
        kind: Kind::Observation,
        stored_unit: "kg",
        units: MASS,
        keywords: &["lean", "fat free", "fat-free", "maigre"],
    },
    Target {
        metric: "muscle_mass",
        label: "Masse musculaire",
        kind: Kind::Observation,
        stored_unit: "kg",
        units: MASS,
        keywords: &["muscle"],
    },
    Target {
        metric: "bone_mass",
        label: "Masse osseuse",
        kind: Kind::Observation,
        stored_unit: "kg",
        units: MASS,
        keywords: &["bone", "osseuse"],
    },
    Target {
        metric: "hydration",
        label: "Hydratation (poids)",
        kind: Kind::Observation,
        stored_unit: "kg",
        units: MASS,
        keywords: &["water", "hydrat"],
    },
    Target {
        metric: "bmi",
        label: "IMC",
        kind: Kind::Observation,
        stored_unit: "kg/m2",
        units: &[unit("kg/m2", "kg/m²")],
        keywords: &["bmi", "imc"],
    },
    Target {
        metric: "spo2",
        label: "Saturation en oxygène",
        kind: Kind::Observation,
        stored_unit: "%",
        units: PERCENT,
        keywords: &["spo2", "oxygen", "oxygène", "saturation"],
    },
    Target {
        metric: "body_temperature",
        label: "Température corporelle",
        kind: Kind::Observation,
        stored_unit: "degC",
        units: &[unit("degC", "°C"), unit("degF", "°F")],
        keywords: &["temperature", "température"],
    },
    Target {
        metric: "resting_heart_rate",
        label: "Fréquence cardiaque au repos",
        kind: Kind::Observation,
        stored_unit: "bpm",
        units: &[unit("bpm", "battements/min")],
        keywords: &["resting", "repos"],
    },
    Target {
        metric: "blood_glucose",
        label: "Glycémie",
        kind: Kind::Observation,
        stored_unit: "mmol/L",
        units: &[unit("mmol/L", "mmol/L"), unit("mg/dL", "mg/dL")],
        keywords: &["glucose", "glycémie", "glycemie"],
    },
    Target {
        metric: "steps",
        label: "Pas (total du jour)",
        kind: Kind::Observation,
        stored_unit: "count",
        units: &[unit("count", "pas")],
        keywords: &["steps", "pas"],
    },
    Target {
        metric: "systolic",
        label: "Tension systolique",
        kind: Kind::Systolic,
        stored_unit: "mmHg",
        units: &[unit("mmHg", "mmHg")],
        keywords: &["systol", "sys"],
    },
    Target {
        metric: "diastolic",
        label: "Tension diastolique",
        kind: Kind::Diastolic,
        stored_unit: "mmHg",
        units: &[unit("mmHg", "mmHg")],
        keywords: &["diastol", "dia"],
    },
];

pub fn find(metric: &str) -> Option<&'static Target> {
    TARGETS.iter().find(|t| t.metric == metric)
}

pub fn convert(metric: &str, unit_id: &str, value: f64) -> Option<f64> {
    let target = find(metric)?;
    if !target.units.iter().any(|u| u.id == unit_id) {
        return None;
    }
    match target.stored_unit {
        "kg" => mass_kg(value, unit_id),
        "%" => Some(if unit_id == "fraction" {
            value * 100.0
        } else {
            value
        }),
        "degC" => temperature_c(value, unit_id),
        "mmol/L" => glucose_mmol_l(value, unit_id),
        _ => Some(value),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct UnitInfo {
    pub id: &'static str,
    pub label: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct TargetInfo {
    pub metric: &'static str,
    pub label: &'static str,
    pub units: Vec<UnitInfo>,
}

pub fn catalogue() -> Vec<TargetInfo> {
    TARGETS
        .iter()
        .map(|t| TargetInfo {
            metric: t.metric,
            label: t.label,
            units: t
                .units
                .iter()
                .map(|u| UnitInfo {
                    id: u.id,
                    label: u.label,
                })
                .collect(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn units_are_converted_to_the_stored_unit() {
        assert!((convert("weight", "lb", 160.0).unwrap() - 72.5748).abs() < 1e-3);
        assert_eq!(convert("fat_ratio", "fraction", 0.185), Some(18.5));
        assert_eq!(convert("fat_ratio", "%", 18.5), Some(18.5));
        assert!((convert("body_temperature", "degF", 98.6).unwrap() - 37.0).abs() < 1e-9);
        assert!((convert("blood_glucose", "mg/dL", 99.0).unwrap() - 5.4955).abs() < 1e-3);
        assert_eq!(convert("blood_glucose", "mmol/L", 5.5), Some(5.5));
        assert_eq!(convert("steps", "count", 4200.0), Some(4200.0));
    }

    #[test]
    fn a_unit_foreign_to_the_metric_or_an_unknown_metric_is_refused() {
        assert_eq!(convert("weight", "mmol/L", 70.0), None);
        assert_eq!(convert("inconnue", "kg", 70.0), None);
    }

    #[test]
    fn every_target_lists_its_stored_unit_among_its_own() {
        for t in TARGETS {
            assert!(!t.units.is_empty(), "{}", t.metric);
            assert!(
                convert(t.metric, t.units[0].id, 1.0).is_some(),
                "{}",
                t.metric
            );
        }
    }
}
