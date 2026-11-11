use super::dates::{local_day, parse, utc_text};
use super::items::RawWorkout;
use super::units::{distance_m, duration_s, energy_kcal};
use crate::sources::import::fingerprint::record_id;
use crate::sources::import::rows::{Skipped, WorkoutRow};

const PREFIX: &str = "HKWorkoutActivityType";

fn known_label(kind: &str) -> Option<&'static str> {
    Some(match kind {
        "Running" => "Course",
        "Walking" => "Marche",
        "Cycling" => "Velo",
        "Swimming" => "Natation",
        "Hiking" => "Randonnee",
        "Yoga" => "Yoga",
        "TraditionalStrengthTraining" | "FunctionalStrengthTraining" => "Musculation",
        "HighIntensityIntervalTraining" => "HIIT",
        "Elliptical" => "Elliptique",
        "Rowing" => "Rameur",
        "Dance" | "SocialDance" => "Danse",
        "Pilates" => "Pilates",
        "CoreTraining" => "Gainage",
        _ => return None,
    })
}

fn humanize(kind: &str) -> String {
    let mut out = String::new();
    for (i, c) in kind.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            out.push(' ');
            out.extend(c.to_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

pub fn label_of(activity: &str) -> String {
    let kind = activity.strip_prefix(PREFIX).unwrap_or(activity);
    known_label(kind).map_or_else(|| humanize(kind), str::to_string)
}

fn number(text: &str) -> Option<f64> {
    text.trim().parse::<f64>().ok().filter(|v| v.is_finite())
}

fn stat<'a>(w: &'a RawWorkout, needle: &str) -> Option<&'a super::items::RawStat> {
    w.stats.iter().find(|s| s.kind.contains(needle))
}

fn distance_of(w: &RawWorkout) -> Option<f64> {
    if let Some(v) = number(&w.distance) {
        return distance_m(v, &w.distance_unit);
    }
    let s = stat(w, "Distance")?;
    distance_m(number(&s.sum)?, &s.unit)
}

fn calories_of(w: &RawWorkout) -> Option<f64> {
    if let Some(v) = number(&w.energy) {
        return energy_kcal(v, &w.energy_unit);
    }
    let s = stat(w, "ActiveEnergyBurned")?;
    energy_kcal(number(&s.sum)?, &s.unit)
}

pub fn workout_row(w: &RawWorkout, skipped: &mut Skipped) -> Option<WorkoutRow> {
    let (Some(start), Some(end)) = (parse(&w.start), parse(&w.end)) else {
        skipped.add("séance : date illisible");
        return None;
    };
    let span = (end.utc - start.utc).num_seconds().max(0);
    let duration = number(&w.duration)
        .and_then(|v| duration_s(v, &w.duration_unit))
        .map_or(span, |s| s.round() as i64);
    Some(WorkoutRow {
        date: local_day(&start),
        external_id: record_id(
            "ah",
            &w.activity,
            start.utc,
            end.utc,
            &w.duration,
            &w.origin,
        ),
        start_at: utc_text(start.utc),
        end_at: utc_text(end.utc),
        category_label: label_of(&w.activity),
        duration,
        distance: distance_of(w),
        calories: calories_of(w),
        device: w.origin.clone(),
        ..WorkoutRow::default()
    })
}

#[cfg(test)]
mod tests {
    use super::super::items::RawStat;
    use super::*;

    fn run() -> RawWorkout {
        RawWorkout {
            activity: "HKWorkoutActivityTypeRunning".into(),
            origin: "Apple Watch".into(),
            start: "2026-10-25 18:00:00 +0200".into(),
            end: "2026-10-25 18:30:30 +0200".into(),
            duration: "30.5".into(),
            duration_unit: "min".into(),
            distance: "3.1".into(),
            distance_unit: "mi".into(),
            energy: "320".into(),
            energy_unit: "kcal".into(),
            stats: Vec::new(),
        }
    }

    #[test]
    fn a_workout_converts_its_units_and_keeps_its_local_day() {
        let mut skipped = Skipped::default();
        let row = workout_row(&run(), &mut skipped).expect("séance");
        assert_eq!(row.category_label, "Course");
        assert_eq!(row.duration, 1830);
        assert!((row.distance.unwrap() - 4988.97).abs() < 0.1);
        assert_eq!(row.calories, Some(320.0));
        assert_eq!(row.date, "2026-10-25");
        assert_eq!(row.start_at, "2026-10-25T16:00:00Z");
        assert_eq!(row.device, "Apple Watch");
        assert_eq!(skipped.total(), 0);
    }

    #[test]
    fn a_recent_export_carries_distance_and_energy_in_statistics() {
        let mut w = run();
        w.distance.clear();
        w.energy.clear();
        w.duration.clear();
        w.stats = vec![
            RawStat {
                kind: "HKQuantityTypeIdentifierDistanceWalkingRunning".into(),
                sum: "5.2".into(),
                unit: "km".into(),
            },
            RawStat {
                kind: "HKQuantityTypeIdentifierActiveEnergyBurned".into(),
                sum: "1255".into(),
                unit: "kJ".into(),
            },
        ];
        let row = workout_row(&w, &mut Skipped::default()).expect("séance");
        assert_eq!(row.distance, Some(5200.0));
        assert!((row.calories.unwrap() - 299.95).abs() < 0.1);
        assert_eq!(row.duration, 1830);
    }

    #[test]
    fn labels_are_french_when_known_and_readable_otherwise() {
        assert_eq!(label_of("HKWorkoutActivityTypeCycling"), "Velo");
        assert_eq!(label_of("HKWorkoutActivityTypeTableTennis"), "Table tennis");
        assert_eq!(label_of("HKWorkoutActivityTypeOther"), "Other");
    }

    #[test]
    fn a_workout_without_a_readable_date_is_counted_not_written() {
        let mut w = run();
        w.start = "hier".into();
        let mut skipped = Skipped::default();
        assert!(workout_row(&w, &mut skipped).is_none());
        assert_eq!(skipped.total(), 1);
    }

    #[test]
    fn the_same_workout_always_has_the_same_id() {
        let a = workout_row(&run(), &mut Skipped::default()).unwrap();
        let b = workout_row(&run(), &mut Skipped::default()).unwrap();
        assert_eq!(a.external_id, b.external_id);
    }
}
