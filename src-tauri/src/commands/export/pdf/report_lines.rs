use super::display::{fr_date, fr_duration, fr_measure, fr_number};
use super::fetch::PdfData;

/// Une section du rapport : son titre et ses lignes deja mises en forme.
pub(super) struct ReportSection {
    pub(super) title: &'static str,
    pub(super) lines: Vec<String>,
}

fn opt_int(value: Option<i64>, unit: &str) -> String {
    value.map_or_else(|| "-".to_string(), |v| format!("{v} {unit}"))
}

fn ecg_label(status: Option<i64>) -> &'static str {
    match status {
        Some(0) => "Normal",
        Some(1) => "AFib",
        Some(2) => "Inconclusif",
        Some(3) => "Trop court",
        _ => "-",
    }
}

fn section_for(data: &PdfData, category: &str) -> Option<ReportSection> {
    let (title, lines): (&'static str, Vec<String>) = match category {
        "body_measures" => ("Composition corporelle", body_measures_lines(data)),
        "blood_pressure" => ("Tension artérielle", blood_pressure_lines(data)),
        "sleep" => ("Sommeil", sleep_lines(data)),
        "activity" => ("Activité", activity_lines(data)),
        "measurements" => ("Mensurations", measurements_lines(data)),
        "workouts" => ("Entraînements", workouts_lines(data)),
        "heart" => ("Cœur et ECG", heart_lines(data)),
        _ => return None,
    };
    Some(ReportSection { title, lines })
}

fn body_measures_lines(data: &PdfData) -> Vec<String> {
    data.bm_rows
        .iter()
        .map(|r| {
            format!(
                "{}   Poids {}   Graisse {}   Muscle {}   Os {}   Eau {}   IMC {}",
                fr_date(&r.measured_at),
                fr_measure(r.weight, "kg"),
                fr_measure(r.fat_ratio, "%"),
                fr_measure(r.muscle_mass, "kg"),
                fr_measure(r.bone_mass, "kg"),
                fr_measure(r.hydration, "kg"),
                r.bmi.map_or_else(|| "-".to_string(), |v| fr_number(v, 1)),
            )
        })
        .collect()
}

fn blood_pressure_lines(data: &PdfData) -> Vec<String> {
    data.bp_rows
        .iter()
        .map(|r| {
            format!(
                "{}   {}/{} mmHg   Pouls {}",
                fr_date(&r.measured_at),
                r.systolic,
                r.diastolic,
                opt_int(r.heart_rate, "bpm"),
            )
        })
        .collect()
}

fn sleep_lines(data: &PdfData) -> Vec<String> {
    data.sleep_rows
        .iter()
        .map(|r| {
            format!(
                "{}   {}   Score {}",
                fr_date(&r.date),
                r.duration_total
                    .map_or_else(|| "-".to_string(), |s| fr_duration(s as f64)),
                r.sleep_score
                    .map_or_else(|| "-".to_string(), |v| format!("{v}/100")),
            )
        })
        .collect()
}

fn activity_lines(data: &PdfData) -> Vec<String> {
    data.activity_rows
        .iter()
        .map(|r| {
            format!(
                "{}   {}   {}   {}",
                fr_date(&r.date),
                opt_int(r.steps, "pas"),
                fr_measure(r.distance.map(|m| m / 1000.0), "km"),
                fr_measure(r.calories_active, "kcal"),
            )
        })
        .collect()
}

fn measurements_lines(data: &PdfData) -> Vec<String> {
    data.meas_rows
        .iter()
        .map(|r| {
            format!(
                "{}   {} : {} {}",
                fr_date(&r.measured_at),
                r.label,
                fr_number(r.value, 1),
                r.unit,
            )
        })
        .collect()
}

fn workouts_lines(data: &PdfData) -> Vec<String> {
    data.workout_rows
        .iter()
        .map(|r| {
            format!(
                "{}   {}   {}   {}",
                fr_date(&r.date),
                r.category_label.as_deref().unwrap_or("-"),
                r.duration
                    .map_or_else(|| "-".to_string(), |s| fr_duration(s as f64)),
                fr_measure(r.calories, "kcal"),
            )
        })
        .collect()
}

fn heart_lines(data: &PdfData) -> Vec<String> {
    data.heart_rows
        .iter()
        .map(|r| {
            format!(
                "{}   {}   {}",
                fr_date(&r.timestamp),
                ecg_label(r.ecg_status),
                opt_int(r.heart_rate, "bpm"),
            )
        })
        .collect()
}

/// Sections du rapport, dans l'ordre des categories demandees ; une categorie
/// inconnue est ignoree.
pub(super) fn report_sections(data: &PdfData, categories: &[String]) -> Vec<ReportSection> {
    categories
        .iter()
        .filter_map(|c| section_for(data, c))
        .collect()
}
