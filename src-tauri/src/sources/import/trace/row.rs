use super::stats::Summary;
use crate::sources::import::fingerprint::record_id;
use crate::sources::import::rows::WorkoutRow;
use crate::sources::import::time::{local_day, utc_text, Stamp};

pub struct Draft<'a> {
    pub prefix: &'a str,
    pub kind: &'a str,
    pub sport: &'a str,
    pub device: &'a str,
    pub summary: Summary,
    pub duration: Option<i64>,
    pub distance: Option<f64>,
    pub calories: Option<f64>,
}

pub fn sport_label(raw: &str) -> String {
    let key = raw.trim().to_lowercase();
    match key.as_str() {
        "running" | "run" | "trail_running" | "treadmill_running" => "Course".into(),
        "cycling" | "biking" | "bike" | "ride" | "road_biking" | "mountain_biking" => "Velo".into(),
        "walking" | "walk" => "Marche".into(),
        "hiking" | "hike" => "Randonnee".into(),
        "swimming" | "swim" | "lap_swimming" | "open_water_swimming" => "Natation".into(),
        "rowing" => "Rameur".into(),
        "yoga" => "Yoga".into(),
        "training" | "fitness_equipment" | "strength_training" => "Entrainement".into(),
        "" | "other" | "generic" => "Activite".into(),
        _ => {
            let mut chars = key.replace('_', " ").chars().collect::<Vec<_>>();
            if let Some(first) = chars.first_mut() {
                *first = first.to_uppercase().next().unwrap_or(*first);
            }
            chars.into_iter().collect()
        }
    }
}

pub fn workout_row(d: &Draft<'_>) -> WorkoutRow {
    let s = &d.summary;
    let elapsed = (s.end - s.start).num_seconds().max(0);
    let distance = d.distance.filter(|v| *v > 0.0);
    let signature = format!("{}|{}", s.timed_points, distance.unwrap_or(0.0).round());
    WorkoutRow {
        date: local_day(&Stamp::on_this_computer(s.start)),
        external_id: record_id(d.prefix, d.kind, s.start, s.end, &signature, d.sport),
        start_at: utc_text(s.start),
        end_at: utc_text(s.end),
        category_label: sport_label(d.sport),
        duration: d.duration.filter(|v| *v > 0).unwrap_or(elapsed),
        distance,
        calories: d.calories.filter(|v| *v > 0.0),
        elevation: Some(s.climb).filter(|v| *v > 0.0),
        hr_average: s.hr_average,
        hr_min: s.hr_min,
        hr_max: s.hr_max,
        device: d.device.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    fn summary() -> Summary {
        Summary {
            start: Utc.with_ymd_and_hms(2026, 10, 25, 10, 0, 0).unwrap(),
            end: Utc.with_ymd_and_hms(2026, 10, 25, 10, 30, 0).unwrap(),
            timed_points: 90,
            distance: 5000.0,
            climb: 0.0,
            hr_average: Some(140),
            hr_min: Some(100),
            hr_max: Some(170),
        }
    }

    #[test]
    fn labels_are_french_when_known_and_readable_otherwise() {
        assert_eq!(sport_label("Running"), "Course");
        assert_eq!(sport_label("Biking"), "Velo");
        assert_eq!(sport_label("other"), "Activite");
        assert_eq!(
            sport_label("stand_up_paddleboarding"),
            "Stand up paddleboarding"
        );
    }

    #[test]
    fn a_row_takes_the_elapsed_time_without_a_timer_and_hides_zeros() {
        let row = workout_row(&Draft {
            prefix: "gpx",
            kind: "gpx",
            sport: "running",
            device: "Montre",
            summary: summary(),
            duration: None,
            distance: Some(0.0),
            calories: Some(0.0),
        });
        assert_eq!(row.duration, 1800);
        assert_eq!(row.distance, None);
        assert_eq!(row.calories, None);
        assert_eq!(row.elevation, None);
        assert!(row.external_id.starts_with("gpx:"));
        assert_eq!(row.start_at, "2026-10-25T10:00:00Z");
    }

    #[test]
    fn the_same_track_has_the_same_id_and_another_distance_another_id() {
        let draft = |distance| Draft {
            prefix: "gpx",
            kind: "gpx",
            sport: "running",
            device: "Montre",
            summary: summary(),
            duration: Some(1700),
            distance: Some(distance),
            calories: None,
        };
        let a = workout_row(&draft(5000.0));
        assert_eq!(a.external_id, workout_row(&draft(5000.0)).external_id);
        assert_ne!(a.external_id, workout_row(&draft(5200.0)).external_id);
        assert_eq!(a.duration, 1700);
    }
}
