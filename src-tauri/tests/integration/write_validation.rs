use synk_lib::commands::alerts::{add_alert_inner, AddAlertInput};
use synk_lib::commands::journal::JournalInput;
use synk_lib::commands::labs::AddLabResultInput;
use synk_lib::commands::manual_measurements::{validate_measurement_goal, AddMeasurementTypeInput};
use synk_lib::commands::medications::{AddMedicationInput, LogMedicationInput};
use synk_lib::error::AppError;
use synk_lib::validation::check_limit;

fn lab(value: f64, min: Option<f64>, max: Option<f64>) -> AddLabResultInput {
    AddLabResultInput {
        date: "2026-10-03".into(),
        test_name: "Glycemie".into(),
        value,
        unit: Some("g/L".into()),
        reference_min: min,
        reference_max: max,
        notes: None,
    }
}

fn alert(metric_key: &str, condition: &str, threshold: f64, label: &str) -> AddAlertInput {
    AddAlertInput {
        metric_key: metric_key.into(),
        condition: condition.into(),
        threshold,
        label: label.into(),
    }
}

fn is_validation<T: std::fmt::Debug>(r: Result<T, AppError>) -> bool {
    matches!(r, Err(AppError::Validation(_)))
}

#[test]
fn lab_results_are_checked() {
    assert!(lab(0.95, Some(0.7), Some(1.1)).validate().is_ok());
    assert!(is_validation(lab(f64::NAN, None, None).validate()));
    assert!(is_validation(lab(1.0, Some(2.0), Some(1.0)).validate()));
    let mut no_name = lab(1.0, None, None);
    no_name.test_name = "   ".into();
    assert!(is_validation(no_name.validate()));
    let mut bad_date = lab(1.0, None, None);
    bad_date.date = "03/10/2026".into();
    assert!(is_validation(bad_date.validate()));
}

#[test]
fn journal_entries_are_checked() {
    let ok = JournalInput {
        date: "2026-10-03".into(),
        content: "Bonne nuit".into(),
        tags: None,
    };
    assert!(ok.validate().is_ok());
    let empty = JournalInput {
        date: "2026-10-03".into(),
        content: " ".into(),
        tags: None,
    };
    assert!(is_validation(empty.validate()));
    let long_tags = JournalInput {
        date: "2026-10-03".into(),
        content: "x".into(),
        tags: Some("t".repeat(201)),
    };
    assert!(is_validation(long_tags.validate()));
}

#[test]
fn medications_are_checked() {
    let ok = AddMedicationInput {
        name: "Paracetamol".into(),
        dosage: Some("1 g".into()),
        frequency: None,
        start_date: Some("2026-10-01".into()),
        end_date: Some("2026-10-05".into()),
        notes: None,
    };
    assert!(ok.validate().is_ok());
    let reversed = AddMedicationInput {
        end_date: Some("2026-09-01".into()),
        ..ok
    };
    assert!(is_validation(reversed.validate()));
    let log = LogMedicationInput {
        medication_id: 1,
        taken_at: "pas une date".into(),
        skipped: false,
        notes: None,
    };
    assert!(is_validation(log.validate()));
}

#[test]
fn measurement_types_goals_and_theme_are_checked() {
    let ok = AddMeasurementTypeInput {
        slug: "tour_mollet".into(),
        label: "Tour de mollet".into(),
        unit: "cm".into(),
    };
    assert!(ok.validate().is_ok());
    let bad_slug = AddMeasurementTypeInput {
        slug: "Tour Mollet".into(),
        label: "x".into(),
        unit: "cm".into(),
    };
    assert!(is_validation(bad_slug.validate()));
    assert!(validate_measurement_goal(Some(80.0), Some("decrease")).is_ok());
    assert!(is_validation(validate_measurement_goal(
        Some(80.0),
        Some("down")
    )));
    assert!(is_validation(validate_measurement_goal(
        Some(f64::INFINITY),
        Some("decrease")
    )));
}

#[test]
fn list_limits_are_bounded() {
    assert_eq!(check_limit(None, 50).unwrap(), 50);
    assert_eq!(check_limit(Some(1000), 50).unwrap(), 1000);
    assert!(is_validation(check_limit(Some(0), 50)));
    assert!(is_validation(check_limit(Some(-1), 50)));
    assert!(is_validation(check_limit(Some(1001), 50)));
}

#[tokio::test]
async fn alerts_accept_the_nine_metrics_of_the_form_and_refuse_the_rest() {
    let pool = crate::common::memory_pool().await;
    for slug in [
        "weight",
        "fat_ratio",
        "muscle_mass",
        "bmi",
        "systolic",
        "diastolic",
        "heart_rate",
        "sleep_score",
        "steps",
    ] {
        add_alert_inner(&pool, &alert(slug, "above", 1.0, "seuil"))
            .await
            .unwrap_or_else(|e| panic!("{slug} refuse : {e}"));
    }
    assert!(is_validation(
        add_alert_inner(&pool, &alert("inconnue", "above", 1.0, "x")).await
    ));
    assert!(is_validation(
        add_alert_inner(&pool, &alert("weight", "equal", 1.0, "x")).await
    ));
    assert!(is_validation(
        add_alert_inner(&pool, &alert("weight", "above", f64::NAN, "x")).await
    ));
    assert!(is_validation(
        add_alert_inner(&pool, &alert("weight", "above", 1.0, "  ")).await
    ));
}

#[tokio::test]
async fn a_negative_limit_no_longer_means_no_limit() {
    let pool = crate::common::memory_pool().await;
    let r = synk_lib::services::error_log::get_error_log(&pool, -1, None).await;
    assert!(is_validation(r));
}
