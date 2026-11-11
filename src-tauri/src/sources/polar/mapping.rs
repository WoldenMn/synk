use std::collections::BTreeMap;

use chrono::{DateTime, Duration, FixedOffset, NaiveDate, TimeZone, Utc};

use super::duration::iso_seconds;
use super::types::{Activity, Exercise, Night, Recharge};
use crate::sources::{Observation, SourceId};

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
    pub hr_average: Option<i64>,
    pub hr_min: Option<i64>,
    pub rr_average: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActivityRow {
    pub date: String,
    pub external_id: String,
    pub steps: Option<i64>,
    pub distance: Option<f64>,
    pub calories_active: Option<f64>,
    pub calories_total: Option<f64>,
    pub duration_active: Option<i64>,
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

fn utc_text(at: DateTime<Utc>) -> String {
    at.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn whole(value: Option<f64>) -> Option<i64> {
    value.filter(|v| v.is_finite()).map(|v| v.round() as i64)
}

fn noon(day: NaiveDate) -> DateTime<Utc> {
    Utc.from_utc_datetime(&day.and_hms_opt(12, 0, 0).unwrap_or_default())
}

fn heart_rate_stats(samples: &BTreeMap<String, f64>) -> (Option<i64>, Option<i64>) {
    let values: Vec<f64> = samples
        .values()
        .copied()
        .filter(|v| v.is_finite())
        .collect();
    if values.is_empty() {
        return (None, None);
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    (whole(Some(mean)), whole(Some(min)))
}

fn night_row(night: &Night) -> SleepRow {
    let stages = [
        night.light_sleep,
        night.deep_sleep,
        night.rem_sleep,
        night.unrecognized_sleep_stage,
    ];
    let total = stages
        .iter()
        .any(Option::is_some)
        .then(|| stages.iter().flatten().sum::<i64>());
    let in_bed = match (night.sleep_start_time, night.sleep_end_time) {
        (Some(start), Some(end)) => Some((end - start).num_seconds()).filter(|s| *s > 0),
        _ => None,
    };
    let efficiency = match (total, in_bed) {
        (Some(asleep), Some(bed)) => {
            Some(((asleep as f64 / bed as f64 * 1000.0).round() / 10.0).min(100.0))
        }
        _ => None,
    };
    let (hr_average, hr_min) = night
        .heart_rate_samples
        .as_ref()
        .map_or((None, None), heart_rate_stats);
    SleepRow {
        date: night.date.to_string(),
        external_id: format!("sleep:{}", night.date),
        start_at: night
            .sleep_start_time
            .map(|t| utc_text(t.with_timezone(&Utc))),
        end_at: night
            .sleep_end_time
            .map(|t| utc_text(t.with_timezone(&Utc))),
        duration_total: total,
        duration_deep: night.deep_sleep,
        duration_light: night.light_sleep,
        duration_rem: night.rem_sleep,
        duration_awake: night.total_interruption_duration,
        total_timeinbed: in_bed,
        sleep_score: night.sleep_score,
        sleep_efficiency: efficiency,
        hr_average,
        hr_min,
        rr_average: None,
    }
}

pub fn sleep_rows(nights: &[Night], recharges: &[Recharge]) -> Vec<SleepRow> {
    let mut rows: BTreeMap<NaiveDate, SleepRow> = nights
        .iter()
        .map(|night| (night.date, night_row(night)))
        .collect();
    for recharge in recharges {
        if let (Some(row), Some(rate)) = (
            rows.get_mut(&recharge.date),
            recharge.breathing_rate_avg.filter(|v| v.is_finite()),
        ) {
            row.rr_average = Some(rate);
        }
    }
    rows.into_values().collect()
}

fn observation(metric: &str, day: NaiveDate, value: f64) -> Observation {
    Observation {
        metric: metric.to_string(),
        measured_at: noon(day),
        value,
        unit: if metric == "hrv" { "ms" } else { "pts" }.to_string(),
        source: SourceId::POLAR,
        external_id: Some(format!("{metric}:{day}")),
        device: None,
    }
}

pub fn recharge_observations(recharges: &[Recharge]) -> Vec<Observation> {
    recharges
        .iter()
        .flat_map(|r| {
            [
                r.heart_rate_variability_avg
                    .filter(|v| *v > 0.0)
                    .map(|v| observation("hrv", r.date, v)),
                r.ans_charge.map(|v| observation("ans_charge", r.date, v)),
                r.nightly_recharge_status
                    .map(|v| observation("nightly_recharge_status", r.date, v)),
            ]
        })
        .flatten()
        .collect()
}

pub fn activity_rows(activities: &[Activity]) -> Vec<ActivityRow> {
    let by_day: BTreeMap<String, ActivityRow> = activities
        .iter()
        .filter_map(|a| {
            let day = a.start_time.get(..10)?.parse::<NaiveDate>().ok()?;
            Some((
                day.to_string(),
                ActivityRow {
                    date: day.to_string(),
                    external_id: format!("activity:{day}"),
                    steps: whole(a.steps),
                    distance: a.distance_from_steps,
                    calories_active: a.active_calories,
                    calories_total: a.calories,
                    duration_active: whole(a.active_duration.as_deref().and_then(iso_seconds)),
                },
            ))
        })
        .collect();
    by_day.into_values().collect()
}

pub fn workout_rows(exercises: &[Exercise]) -> Vec<WorkoutRow> {
    exercises
        .iter()
        .map(|e| {
            let offset = FixedOffset::east_opt(
                i32::try_from(e.start_time_utc_offset.unwrap_or(0) * 60).unwrap_or(0),
            )
            .unwrap_or_else(|| FixedOffset::east_opt(0).expect("UTC"));
            let start = offset
                .from_local_datetime(&e.start_time)
                .single()
                .map_or_else(|| e.start_time.and_utc(), |t| t.with_timezone(&Utc));
            let seconds = e
                .duration
                .as_deref()
                .and_then(iso_seconds)
                .map_or(0, |s| s.round() as i64)
                .max(0);
            let label = e
                .detailed_sport_info
                .as_deref()
                .or(e.sport.as_deref())
                .unwrap_or("other")
                .to_lowercase();
            WorkoutRow {
                date: e.start_time.date().to_string(),
                external_id: e.id.clone(),
                start_at: utc_text(start),
                end_at: utc_text(start + Duration::seconds(seconds)),
                category_label: label,
                duration: seconds,
                distance: e.distance,
                calories: e.calories,
                hr_average: whole(e.heart_rate.as_ref().and_then(|h| h.average)),
                hr_max: whole(e.heart_rate.as_ref().and_then(|h| h.maximum)),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nights(json: &str) -> Vec<Night> {
        serde_json::from_str(json).expect("nuits")
    }

    fn recharges(json: &str) -> Vec<Recharge> {
        serde_json::from_str(json).expect("recharges")
    }

    const NIGHT: &str = r#"[{"date":"2026-10-24","sleep_start_time":"2026-10-23T23:30:00+02:00","sleep_end_time":"2026-10-24T07:10:00+02:00",
        "light_sleep":13500,"deep_sleep":5400,"rem_sleep":6300,"unrecognized_sleep_stage":300,"total_interruption_duration":2100,
        "sleep_score":83,"heart_rate_samples":{"00:41":58,"00:46":52,"00:51":50}}]"#;

    #[test]
    fn a_night_gives_one_row_with_stage_totals_time_in_bed_and_efficiency() {
        let rows = sleep_rows(&nights(NIGHT), &[]);
        assert_eq!(rows.len(), 1);
        let row = &rows[0];
        assert_eq!(row.date, "2026-10-24");
        assert_eq!(row.external_id, "sleep:2026-10-24");
        assert_eq!(row.start_at.as_deref(), Some("2026-10-23T21:30:00Z"));
        assert_eq!(row.end_at.as_deref(), Some("2026-10-24T05:10:00Z"));
        assert_eq!(row.duration_total, Some(25500));
        assert_eq!(row.duration_light, Some(13500));
        assert_eq!(row.duration_deep, Some(5400));
        assert_eq!(row.duration_rem, Some(6300));
        assert_eq!(row.duration_awake, Some(2100));
        assert_eq!(row.total_timeinbed, Some(27600));
        assert_eq!(row.sleep_efficiency, Some(92.4));
        assert_eq!(row.sleep_score, Some(83));
        assert_eq!(row.hr_average, Some(53));
        assert_eq!(row.hr_min, Some(50));
        assert_eq!(row.rr_average, None);
    }

    #[test]
    fn a_night_without_times_or_stages_keeps_what_it_has_and_invents_nothing() {
        let rows = sleep_rows(&nights(r#"[{"date":"2026-10-25","sleep_score":70}]"#), &[]);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].sleep_score, Some(70));
        assert_eq!(rows[0].duration_total, None);
        assert_eq!(rows[0].total_timeinbed, None);
        assert_eq!(rows[0].sleep_efficiency, None);
        assert_eq!(rows[0].start_at, None);
    }

    #[test]
    fn the_breathing_rate_of_the_recharge_joins_its_night_and_a_lone_recharge_makes_no_night() {
        let rows = sleep_rows(
            &nights(NIGHT),
            &recharges(
                r#"[{"date":"2026-10-24","breathing_rate_avg":14.1},{"date":"2026-10-20","breathing_rate_avg":13.0}]"#,
            ),
        );
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].rr_average, Some(14.1));
    }

    #[test]
    fn a_recharge_gives_hrv_ans_charge_and_status_at_noon_utc_with_distinct_external_ids() {
        let obs = recharge_observations(&recharges(
            r#"[{"date":"2026-10-24","heart_rate_variability_avg":28,"ans_charge":-1.5,"nightly_recharge_status":4},
                {"date":"2026-10-25","heart_rate_variability_avg":0}]"#,
        ));
        let lines: Vec<(&str, f64, &str, &str)> = obs
            .iter()
            .map(|o| {
                (
                    o.metric.as_str(),
                    o.value,
                    o.external_id.as_deref().unwrap(),
                    o.unit.as_str(),
                )
            })
            .collect();
        assert_eq!(
            lines,
            vec![
                ("hrv", 28.0, "hrv:2026-10-24", "ms"),
                ("ans_charge", -1.5, "ans_charge:2026-10-24", "pts"),
                (
                    "nightly_recharge_status",
                    4.0,
                    "nightly_recharge_status:2026-10-24",
                    "pts"
                ),
            ]
        );
        assert_eq!(obs[0].source, SourceId::POLAR);
        assert_eq!(
            obs[0].measured_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            "2026-10-24T12:00:00Z"
        );
    }

    #[test]
    fn a_daily_activity_takes_its_day_from_the_start_time_and_its_active_time_from_iso() {
        let docs: Vec<Activity> = serde_json::from_str(
            r#"[{"start_time":"2026-10-24T07:12:00","end_time":"2026-10-24T23:59:59","active_duration":"PT3H11M",
                 "calories":2500,"active_calories":420,"steps":8823,"distance_from_steps":4590.5}]"#,
        )
        .expect("activite");
        let rows = activity_rows(&docs);
        assert_eq!(rows.len(), 1);
        let row = &rows[0];
        assert_eq!(row.date, "2026-10-24");
        assert_eq!(row.external_id, "activity:2026-10-24");
        assert_eq!(row.steps, Some(8823));
        assert_eq!(row.distance, Some(4590.5));
        assert_eq!(row.calories_active, Some(420.0));
        assert_eq!(row.calories_total, Some(2500.0));
        assert_eq!(row.duration_active, Some(11460));
    }

    #[test]
    fn the_same_day_seen_in_two_overlapping_ranges_makes_one_row_and_the_last_wins() {
        let docs: Vec<Activity> = serde_json::from_str(
            r#"[{"start_time":"2026-10-24T07:12:00","steps":100},{"start_time":"2026-10-24T07:12:00","steps":8823}]"#,
        )
        .expect("activite");
        let rows = activity_rows(&docs);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].steps, Some(8823));
    }

    #[test]
    fn an_activity_with_an_unreadable_start_time_is_skipped_not_guessed() {
        let docs: Vec<Activity> =
            serde_json::from_str(r#"[{"start_time":"hier","steps":10}]"#).expect("activite");
        assert!(activity_rows(&docs).is_empty());
    }

    #[test]
    fn an_exercise_becomes_a_utc_window_with_its_label_and_heart_rate() {
        let docs: Vec<Exercise> = serde_json::from_str(
            r#"[{"id":"2AC312F","start_time":"2026-10-24T18:00:00","start_time_utc_offset":120,"duration":"PT45M30S",
                 "calories":512,"distance":8200.5,"sport":"RUNNING","heart_rate":{"average":148,"maximum":171}},
                {"id":"X9","start_time":"2026-10-25T08:00:00","duration":"PT30M","sport":"OTHER","detailed_sport_info":"WATERSPORTS_WATERSKI"}]"#,
        )
        .expect("seances");
        let rows = workout_rows(&docs);
        assert_eq!(rows.len(), 2);
        let run = &rows[0];
        assert_eq!(run.external_id, "2AC312F");
        assert_eq!(run.date, "2026-10-24");
        assert_eq!(run.start_at, "2026-10-24T16:00:00Z");
        assert_eq!(run.end_at, "2026-10-24T16:45:30Z");
        assert_eq!(run.duration, 2730);
        assert_eq!(run.category_label, "running");
        assert_eq!(run.distance, Some(8200.5));
        assert_eq!(run.calories, Some(512.0));
        assert_eq!(run.hr_average, Some(148));
        assert_eq!(run.hr_max, Some(171));
        let ski = &rows[1];
        assert_eq!(ski.category_label, "watersports_waterski");
        assert_eq!(ski.start_at, "2026-10-25T08:00:00Z");
        assert_eq!(ski.hr_average, None);
    }
}
