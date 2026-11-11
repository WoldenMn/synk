use crate::commands::export::pdf::display::{fr_date, fr_measure, fr_number, record_value};
use crate::commands::profile_card::types::ProfileCard;

fn dash(value: Option<String>) -> String {
    value.unwrap_or_else(|| "-".to_string())
}

pub(super) fn body_rows(card: &ProfileCard) -> Vec<(String, String)> {
    vec![
        (
            "Âge".into(),
            card.body
                .age
                .map(|a| format!("{a} ans"))
                .unwrap_or_else(|| "-".into()),
        ),
        ("Sexe".into(), dash(card.body.sex.clone())),
        ("Taille".into(), fr_measure(card.body.height_cm, "cm")),
        ("Poids".into(), fr_measure(card.body.weight_kg, "kg")),
        (
            "IMC".into(),
            card.body
                .bmi
                .map(|v| fr_number(v, 1))
                .unwrap_or_else(|| "-".into()),
        ),
        ("Masse grasse".into(), fr_measure(card.body.fat_ratio, "%")),
        (
            "Masse musculaire".into(),
            fr_measure(card.body.muscle_mass, "kg"),
        ),
        ("Hydratation".into(), fr_measure(card.body.hydration, "kg")),
        (
            "Dernière pesée".into(),
            dash(card.body.measured_at.as_deref().map(fr_date)),
        ),
    ]
}

pub(super) fn key_measurement_rows(card: &ProfileCard) -> Vec<(String, String)> {
    card.key_measurements
        .iter()
        .map(|m| (m.label.clone(), fr_measure(m.value, "cm")))
        .collect()
}

pub(super) fn treatment_rows(card: &ProfileCard) -> Vec<(String, String)> {
    // Une ligne par traitement plutot qu'une liste fusionnee par virgules :
    // plus lisible, et evite de reconstituer un pave de texte qu'il faudrait
    // ensuite re-decouper. Le titre de section porte deja le libelle, chaque
    // ligne ne repete donc pas "Traitements en cours".
    if card.health.active_treatments.is_empty() {
        vec![(String::new(), "-".to_string())]
    } else {
        card.health
            .active_treatments
            .iter()
            .map(|t| (String::new(), t.clone()))
            .collect()
    }
}

pub(super) fn size_rows(card: &ProfileCard) -> Vec<(String, String)> {
    card.sizes
        .iter()
        .map(|s| (s.category.clone(), s.value.clone()))
        .collect()
}

pub(super) fn record_rows(card: &ProfileCard) -> Vec<(String, String)> {
    card.records
        .iter()
        .map(|r| {
            (
                r.label.clone(),
                format!(
                    "{} ({})",
                    record_value(r.value, &r.unit),
                    fr_date(&r.achieved_at)
                ),
            )
        })
        .collect()
}
