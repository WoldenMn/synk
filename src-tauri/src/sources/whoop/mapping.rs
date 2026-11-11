use std::collections::BTreeMap;

use chrono::{DateTime, FixedOffset, NaiveDate, TimeZone, Utc};

use super::types::{Cycle, Recovery, Sleep, SleepScore, Workout};
use crate::sources::{Observation, SourceId};

const KJ_PER_KCAL: f64 = 4.184;
const SCORED: &str = "SCORED";

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SleepRow {
    pub date: String,
    pub external_id: String,
    pub start_at: Option<String>,
    pub end_at: Option<String>,
    pub duration_total: Option<i64>,
    pub duration_deep: Option<i64>,
    pub duration_light: Option<i64>,
    pub duration_rem: Option<i64>,
    pub duration_awake: Option<i64>,
    pub total_timeinbed: Option<i64>,
    pub sleep_score: Option<i64>,
    pub sleep_efficiency: Option<f64>,
    pub rr_average: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActivityRow {
    pub date: String,
    pub external_id: String,
    pub calories_total: Option<f64>,
    pub hr_average: Option<i64>,
    pub hr_max: Option<i64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorkoutRow {
    pub date: String,
    pub external_id: String,
    pub start_at: String,
    pub end_at: String,
    pub category_label: String,
    pub duration: i64,
    pub distance: Option<f64>,
    pub calories: Option<f64>,
    pub hr_average: Option<i64>,
    pub hr_max: Option<i64>,
}

fn utc_offset() -> FixedOffset {
    FixedOffset::east_opt(0).unwrap_or_else(|| FixedOffset::west_opt(0).expect("UTC"))
}

pub fn parse_offset(text: Option<&str>) -> FixedOffset {
    let text = text.unwrap_or("").trim();
    let (sign, rest) = if let Some(rest) = text.strip_prefix('-') {
        (-1, rest)
    } else if let Some(rest) = text.strip_prefix('+') {
        (1, rest)
    } else {
        return utc_offset();
    };
    let mut parts = rest.split(':');
    let hours = parts.next().and_then(|h| h.parse::<i32>().ok());
    let minutes = parts.next().map_or(Some(0), |m| m.parse::<i32>().ok());
    match (hours, minutes) {
        (Some(h), Some(m)) if (0..24).contains(&h) && (0..60).contains(&m) => {
            FixedOffset::east_opt(sign * (h * 3600 + m * 60)).unwrap_or_else(utc_offset)
        }
        _ => utc_offset(),
    }
}

pub fn local_day(at: DateTime<Utc>, offset: Option<&str>) -> NaiveDate {
    at.with_timezone(&parse_offset(offset)).date_naive()
}

fn utc_text(at: DateTime<Utc>) -> String {
    at.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn noon(day: NaiveDate) -> DateTime<Utc> {
    Utc.from_utc_datetime(&day.and_hms_opt(12, 0, 0).unwrap_or_default())
}

fn whole(value: Option<f64>) -> Option<i64> {
    value.filter(|v| v.is_finite()).map(|v| v.round() as i64)
}

fn seconds(milli: Option<f64>) -> Option<i64> {
    whole(milli.map(|m| m / 1000.0))
}

fn kilocalories(kilojoule: Option<f64>) -> Option<f64> {
    kilojoule
        .filter(|v| v.is_finite())
        .map(|kj| (kj / KJ_PER_KCAL).round())
}

fn scored<'a, T>(state: &Option<String>, score: &'a Option<T>) -> Option<&'a T> {
    if state.as_deref() == Some(SCORED) {
        score.as_ref()
    } else {
        None
    }
}

fn asleep(score: &SleepScore) -> Option<f64> {
    let stages = score.stage_summary.as_ref()?;
    let parts = [
        stages.total_light_sleep_time_milli,
        stages.total_slow_wave_sleep_time_milli,
        stages.total_rem_sleep_time_milli,
    ];
    parts
        .iter()
        .any(Option::is_some)
        .then(|| parts.iter().flatten().sum())
}

fn night_row(sleep: &Sleep, score: &SleepScore) -> SleepRow {
    let stages = score.stage_summary.as_ref();
    SleepRow {
        date: local_day(sleep.end, sleep.timezone_offset.as_deref()).to_string(),
        external_id: sleep.id.clone(),
        start_at: Some(utc_text(sleep.start)),
        end_at: Some(utc_text(sleep.end)),
        duration_total: seconds(asleep(score)),
        duration_deep: seconds(stages.and_then(|s| s.total_slow_wave_sleep_time_milli)),
        duration_light: seconds(stages.and_then(|s| s.total_light_sleep_time_milli)),
        duration_rem: seconds(stages.and_then(|s| s.total_rem_sleep_time_milli)),
        duration_awake: seconds(stages.and_then(|s| s.total_awake_time_milli)),
        total_timeinbed: seconds(stages.and_then(|s| s.total_in_bed_time_milli)),
        sleep_score: whole(score.sleep_performance_percentage),
        sleep_efficiency: score.sleep_efficiency_percentage,
        rr_average: score.respiratory_rate,
    }
}

pub fn sleep_rows(sleeps: &[Sleep]) -> Vec<SleepRow> {
    let mut best: BTreeMap<String, (f64, SleepRow)> = BTreeMap::new();
    for sleep in sleeps.iter().filter(|s| !s.nap) {
        let Some(score) = scored(&sleep.score_state, &sleep.score) else {
            continue;
        };
        let row = night_row(sleep, score);
        let length = asleep(score).unwrap_or(0.0);
        let better = best.get(&row.date).is_none_or(|(kept, _)| length > *kept);
        if better {
            best.insert(row.date.clone(), (length, row));
        }
    }
    best.into_values().map(|(_, row)| row).collect()
}

fn observation(metric: &str, unit: &str, day: NaiveDate, doc_id: &str, value: f64) -> Observation {
    Observation {
        metric: metric.to_string(),
        measured_at: noon(day),
        value,
        unit: unit.to_string(),
        source: SourceId::WHOOP,
        external_id: Some(format!("{metric}:{doc_id}")),
        device: None,
    }
}

pub fn recovery_observations(recoveries: &[Recovery], sleeps: &[Sleep]) -> Vec<Observation> {
    let by_id: BTreeMap<&str, &Sleep> = sleeps
        .iter()
        .filter(|s| !s.nap)
        .map(|s| (s.id.as_str(), s))
        .collect();
    recoveries
        .iter()
        .filter_map(|r| {
            let score = scored(&r.score_state, &r.score)?;
            let sleep = by_id.get(r.sleep_id.as_str())?;
            let day = local_day(sleep.end, sleep.timezone_offset.as_deref());
            let id = r.sleep_id.as_str();
            Some([
                score
                    .recovery_score
                    .map(|v| observation("recovery_score", "%", day, id, v)),
                score
                    .resting_heart_rate
                    .map(|v| observation("resting_heart_rate", "bpm", day, id, v)),
                score
                    .hrv_rmssd_milli
                    .map(|v| observation("hrv", "ms", day, id, v)),
                score
                    .spo2_percentage
                    .map(|v| observation("spo2", "%", day, id, v)),
                score
                    .skin_temp_celsius
                    .map(|v| observation("skin_temperature", "degC", day, id, v)),
            ])
        })
        .flatten()
        .flatten()
        .collect()
}

pub fn cycle_rows(cycles: &[Cycle]) -> (Vec<ActivityRow>, Vec<Observation>) {
    let mut rows: BTreeMap<String, ActivityRow> = BTreeMap::new();
    let mut observations = Vec::new();
    let mut ordered: Vec<&Cycle> = cycles.iter().collect();
    ordered.sort_by_key(|c| c.start);
    for cycle in ordered {
        let Some(score) = scored(&cycle.score_state, &cycle.score) else {
            continue;
        };
        let day = local_day(cycle.start, cycle.timezone_offset.as_deref());
        let id = cycle.id.to_string();
        rows.insert(
            day.to_string(),
            ActivityRow {
                date: day.to_string(),
                external_id: id.clone(),
                calories_total: kilocalories(score.kilojoule),
                hr_average: whole(score.average_heart_rate),
                hr_max: whole(score.max_heart_rate),
            },
        );
        if let Some(strain) = score.strain {
            observations.push(observation("strain", "pts", day, &id, strain));
        }
    }
    (rows.into_values().collect(), observations)
}

pub fn workout_rows(workouts: &[Workout]) -> Vec<WorkoutRow> {
    workouts
        .iter()
        .filter_map(|w| {
            let score = scored(&w.score_state, &w.score)?;
            Some(WorkoutRow {
                date: local_day(w.start, w.timezone_offset.as_deref()).to_string(),
                external_id: w.id.clone(),
                start_at: utc_text(w.start),
                end_at: utc_text(w.end),
                category_label: w
                    .sport_name
                    .clone()
                    .filter(|n| !n.trim().is_empty())
                    .unwrap_or_else(|| "whoop".to_string()),
                duration: (w.end - w.start).num_seconds().max(0),
                distance: score.distance_meter.filter(|v| v.is_finite()),
                calories: kilocalories(score.kilojoule),
                hr_average: whole(score.average_heart_rate),
                hr_max: whole(score.max_heart_rate),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sleeps(json: &str) -> Vec<Sleep> {
        serde_json::from_str(json).expect("sommeil")
    }

    const NIGHT: &str = r#"[{
        "id": "sl-test-1", "cycle_id": 9001, "user_id": 1,
        "start": "2026-10-24T22:10:00.000Z", "end": "2026-10-25T06:40:00.000Z",
        "timezone_offset": "+02:00", "nap": false, "score_state": "SCORED",
        "score": {
            "stage_summary": {
                "total_in_bed_time_milli": 30600000, "total_awake_time_milli": 2400000,
                "total_no_data_time_milli": 0, "total_light_sleep_time_milli": 14400000,
                "total_slow_wave_sleep_time_milli": 7200000, "total_rem_sleep_time_milli": 6600000,
                "sleep_cycle_count": 4, "disturbance_count": 9
            },
            "respiratory_rate": 15.5, "sleep_performance_percentage": 88,
            "sleep_consistency_percentage": 80, "sleep_efficiency_percentage": 92.3
        }
    }]"#;

    #[test]
    fn an_offset_text_is_read_and_a_missing_or_broken_one_means_utc() {
        assert_eq!(parse_offset(Some("-05:00")).local_minus_utc(), -18000);
        assert_eq!(parse_offset(Some("+05:30")).local_minus_utc(), 19800);
        assert_eq!(parse_offset(Some("+02:00")).local_minus_utc(), 7200);
        assert_eq!(parse_offset(Some("Z")).local_minus_utc(), 0);
        assert_eq!(parse_offset(None).local_minus_utc(), 0);
        assert_eq!(parse_offset(Some("n importe quoi")).local_minus_utc(), 0);
        assert_eq!(parse_offset(Some("+25:00")).local_minus_utc(), 0);
    }

    #[test]
    fn the_local_day_follows_the_offset_across_midnight() {
        let late = "2026-10-25T23:30:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(local_day(late, Some("+02:00")).to_string(), "2026-10-26");
        assert_eq!(local_day(late, Some("-05:00")).to_string(), "2026-10-25");
        assert_eq!(local_day(late, None).to_string(), "2026-10-25");
    }

    #[test]
    fn a_scored_night_becomes_one_row_dated_by_the_local_wake_up_day() {
        let rows = sleep_rows(&sleeps(NIGHT));
        assert_eq!(rows.len(), 1);
        let row = &rows[0];
        assert_eq!(row.date, "2026-10-25");
        assert_eq!(row.external_id, "sl-test-1");
        assert_eq!(row.start_at.as_deref(), Some("2026-10-24T22:10:00Z"));
        assert_eq!(row.end_at.as_deref(), Some("2026-10-25T06:40:00Z"));
        assert_eq!(row.duration_total, Some(28200));
        assert_eq!(row.duration_deep, Some(7200));
        assert_eq!(row.duration_light, Some(14400));
        assert_eq!(row.duration_rem, Some(6600));
        assert_eq!(row.duration_awake, Some(2400));
        assert_eq!(row.total_timeinbed, Some(30600));
        assert_eq!(row.sleep_score, Some(88));
        assert_eq!(row.sleep_efficiency, Some(92.3));
        assert_eq!(row.rr_average, Some(15.5));
    }

    #[test]
    fn naps_and_unscored_nights_never_become_the_night() {
        let json = NIGHT
            .replace("\"nap\": false", "\"nap\": true")
            .replace("sl-test-1", "sl-nap");
        assert!(sleep_rows(&sleeps(&json)).is_empty());
        let pending = r#"[{"id":"sl-p","start":"2026-10-24T22:00:00.000Z","end":"2026-10-25T06:00:00.000Z","timezone_offset":"+02:00","nap":false,"score_state":"PENDING_SCORE"}]"#;
        assert!(sleep_rows(&sleeps(pending)).is_empty());
        let inconsistent = NIGHT.replace("SCORED", "UNSCORABLE");
        assert!(sleep_rows(&sleeps(&inconsistent)).is_empty());
    }

    #[test]
    fn two_sleeps_ending_the_same_day_keep_the_longer_one() {
        let short = NIGHT
            .replace("sl-test-1", "sl-short")
            .replace("14400000", "3600000");
        let mut all = sleeps(NIGHT);
        all.extend(sleeps(&short));
        let rows = sleep_rows(&all);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].external_id, "sl-test-1");
    }

    fn recoveries(json: &str) -> Vec<Recovery> {
        serde_json::from_str(json).expect("recuperation")
    }

    const RECOVERY: &str = r#"[{
        "cycle_id": 9001, "sleep_id": "sl-test-1", "user_id": 1,
        "created_at": "2026-10-25T06:45:00.000Z", "updated_at": "2026-10-25T06:45:00.000Z",
        "score_state": "SCORED",
        "score": {
            "user_calibrating": false, "recovery_score": 72, "resting_heart_rate": 51,
            "hrv_rmssd_milli": 64.25, "spo2_percentage": 96.5, "skin_temp_celsius": 33.4
        }
    }]"#;

    fn by_metric(observations: &[Observation], metric: &str) -> Vec<(f64, String, String)> {
        observations
            .iter()
            .filter(|o| o.metric == metric)
            .map(|o| {
                (
                    o.value,
                    o.measured_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                    o.external_id.clone().unwrap_or_default(),
                )
            })
            .collect()
    }

    #[test]
    fn a_recovery_is_dated_by_the_sleep_it_names_and_gives_five_observations() {
        let observations = recovery_observations(&recoveries(RECOVERY), &sleeps(NIGHT));
        assert_eq!(observations.len(), 5);
        let at = "2026-10-25T12:00:00Z".to_string();
        assert_eq!(
            by_metric(&observations, "recovery_score"),
            vec![(72.0, at, "recovery_score:sl-test-1".into())]
        );
        assert_eq!(by_metric(&observations, "resting_heart_rate")[0].0, 51.0);
        assert_eq!(by_metric(&observations, "hrv")[0].0, 64.25);
        assert_eq!(by_metric(&observations, "spo2")[0].0, 96.5);
        assert_eq!(by_metric(&observations, "skin_temperature")[0].0, 33.4);
        assert!(observations.iter().all(|o| o.source.as_str() == "whoop"));
    }

    #[test]
    fn a_recovery_without_its_sleep_or_without_a_score_writes_nothing() {
        assert!(recovery_observations(&recoveries(RECOVERY), &[]).is_empty());
        let pending = RECOVERY.replace("SCORED", "PENDING_SCORE");
        assert!(recovery_observations(&recoveries(&pending), &sleeps(NIGHT)).is_empty());
    }

    #[test]
    fn a_recovery_with_missing_fields_keeps_the_present_ones() {
        let sparse = r#"[{"cycle_id":9001,"sleep_id":"sl-test-1","score_state":"SCORED","score":{"recovery_score":40,"resting_heart_rate":55,"hrv_rmssd_milli":30.5}}]"#;
        let observations = recovery_observations(&recoveries(sparse), &sleeps(NIGHT));
        let metrics: Vec<&str> = observations.iter().map(|o| o.metric.as_str()).collect();
        assert_eq!(metrics.len(), 3);
        assert!(!metrics.contains(&"spo2") && !metrics.contains(&"skin_temperature"));
    }

    fn cycles(json: &str) -> Vec<Cycle> {
        serde_json::from_str(json).expect("cycle")
    }

    const CYCLE: &str = r#"[{
        "id": 9001, "user_id": 1, "start": "2026-10-25T06:40:00.000Z", "end": "2026-10-26T06:50:00.000Z",
        "timezone_offset": "+02:00", "score_state": "SCORED",
        "score": {"strain": 12.5, "kilojoule": 9414.0, "average_heart_rate": 68, "max_heart_rate": 160}
    }]"#;

    #[test]
    fn a_cycle_gives_calories_heart_rate_and_a_strain_observation() {
        let (rows, observations) = cycle_rows(&cycles(CYCLE));
        assert_eq!(
            rows,
            vec![ActivityRow {
                date: "2026-10-25".into(),
                external_id: "9001".into(),
                calories_total: Some(2250.0),
                hr_average: Some(68),
                hr_max: Some(160),
            }]
        );
        assert_eq!(
            by_metric(&observations, "strain"),
            vec![(12.5, "2026-10-25T12:00:00Z".into(), "strain:9001".into())]
        );
    }

    #[test]
    fn an_open_cycle_is_written_when_scored_and_skipped_when_not() {
        let open = CYCLE.replace(", \"end\": \"2026-10-26T06:50:00.000Z\"", "");
        let (rows, _) = cycle_rows(&cycles(&open));
        assert_eq!(rows.len(), 1);
        let pending = CYCLE.replace("SCORED", "PENDING_SCORE");
        let (rows, observations) = cycle_rows(&cycles(&pending));
        assert!(rows.is_empty() && observations.is_empty());
    }

    fn workouts(json: &str) -> Vec<Workout> {
        serde_json::from_str(json).expect("entrainement")
    }

    const WORKOUT: &str = r#"[{
        "id": "wk-test-1", "user_id": 1,
        "start": "2026-10-25T16:00:00.000Z", "end": "2026-10-25T16:45:30.000Z",
        "timezone_offset": "+02:00", "sport_name": "running", "sport_id": 0, "score_state": "SCORED",
        "score": {"strain": 9.4, "average_heart_rate": 150, "max_heart_rate": 178,
                  "kilojoule": 1673.6, "percent_recorded": 100, "distance_meter": 7200.5}
    }]"#;

    #[test]
    fn a_workout_keeps_its_uuid_utc_times_sport_and_converted_energy() {
        let rows = workout_rows(&workouts(WORKOUT));
        assert_eq!(
            rows,
            vec![WorkoutRow {
                date: "2026-10-25".into(),
                external_id: "wk-test-1".into(),
                start_at: "2026-10-25T16:00:00Z".into(),
                end_at: "2026-10-25T16:45:30Z".into(),
                category_label: "running".into(),
                duration: 2730,
                distance: Some(7200.5),
                calories: Some(400.0),
                hr_average: Some(150),
                hr_max: Some(178),
            }]
        );
    }

    #[test]
    fn an_unscored_workout_or_one_without_a_sport_name_is_handled() {
        let pending = WORKOUT.replace("SCORED", "PENDING_SCORE");
        assert!(workout_rows(&workouts(&pending)).is_empty());
        let nameless = WORKOUT.replace("\"sport_name\": \"running\", ", "");
        assert_eq!(
            workout_rows(&workouts(&nameless))[0].category_label,
            "whoop"
        );
    }
}
