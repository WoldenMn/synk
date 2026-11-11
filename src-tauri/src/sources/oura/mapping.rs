use std::collections::BTreeMap;

use chrono::{DateTime, FixedOffset, NaiveDate, TimeZone, Utc};

use super::types::{DailyActivity, DailyReadiness, DailySleep, DailySpo2, SleepPeriod, Workout};
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
    pub duration_to_sleep: Option<i64>,
    pub total_timeinbed: Option<i64>,
    pub sleep_score: Option<i64>,
    pub sleep_efficiency: Option<f64>,
    pub hr_average: Option<i64>,
    pub hr_min: Option<i64>,
    pub rr_average: Option<f64>,
    pub average_hrv: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActivityRow {
    pub date: String,
    pub external_id: String,
    pub steps: Option<i64>,
    pub distance: Option<f64>,
    pub calories_active: Option<f64>,
    pub calories_total: Option<f64>,
    pub duration_soft: Option<i64>,
    pub duration_moderate: Option<i64>,
    pub duration_intense: Option<i64>,
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
}

fn utc_text(at: DateTime<FixedOffset>) -> String {
    at.with_timezone(&Utc)
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string()
}

fn whole(value: Option<f64>) -> Option<i64> {
    value.filter(|v| v.is_finite()).map(|v| v.round() as i64)
}

fn noon(day: NaiveDate) -> DateTime<Utc> {
    Utc.from_utc_datetime(&day.and_hms_opt(12, 0, 0).unwrap_or_default())
}

fn main_periods(periods: &[SleepPeriod]) -> BTreeMap<NaiveDate, &SleepPeriod> {
    let mut best: BTreeMap<NaiveDate, &SleepPeriod> = BTreeMap::new();
    for period in periods
        .iter()
        .filter(|p| p.kind.as_deref() == Some("long_sleep"))
    {
        let length = period.total_sleep_duration.unwrap_or(0.0);
        let better = best
            .get(&period.day)
            .is_none_or(|kept| length > kept.total_sleep_duration.unwrap_or(0.0));
        if better {
            best.insert(period.day, period);
        }
    }
    best
}

fn night_row(period: &SleepPeriod) -> SleepRow {
    SleepRow {
        date: period.day.to_string(),
        external_id: period.id.clone(),
        start_at: Some(utc_text(period.bedtime_start)),
        end_at: Some(utc_text(period.bedtime_end)),
        duration_total: whole(period.total_sleep_duration),
        duration_deep: whole(period.deep_sleep_duration),
        duration_light: whole(period.light_sleep_duration),
        duration_rem: whole(period.rem_sleep_duration),
        duration_awake: whole(period.awake_time),
        duration_to_sleep: whole(period.latency),
        total_timeinbed: whole(period.time_in_bed),
        sleep_efficiency: period.efficiency,
        hr_average: whole(period.average_heart_rate),
        hr_min: whole(period.lowest_heart_rate),
        rr_average: period.average_breath,
        average_hrv: period.average_hrv,
        ..SleepRow::default()
    }
}

pub fn sleep_rows(periods: &[SleepPeriod], scores: &[DailySleep]) -> Vec<SleepRow> {
    let mut rows: BTreeMap<NaiveDate, SleepRow> = main_periods(periods)
        .into_iter()
        .map(|(day, period)| (day, night_row(period)))
        .collect();
    for doc in scores.iter().filter(|d| d.score.is_some()) {
        let row = rows.entry(doc.day).or_insert_with(|| SleepRow {
            date: doc.day.to_string(),
            external_id: doc.id.clone(),
            ..SleepRow::default()
        });
        row.sleep_score = whole(doc.score);
    }
    rows.into_values().collect()
}

pub fn activity_rows(docs: &[DailyActivity]) -> Vec<ActivityRow> {
    docs.iter()
        .map(|d| ActivityRow {
            date: d.day.to_string(),
            external_id: d.id.clone(),
            steps: whole(d.steps),
            distance: d.equivalent_walking_distance,
            calories_active: d.active_calories,
            calories_total: d.total_calories,
            duration_soft: whole(d.low_activity_time),
            duration_moderate: whole(d.medium_activity_time),
            duration_intense: whole(d.high_activity_time),
            duration_active: match (d.medium_activity_time, d.high_activity_time) {
                (None, None) => None,
                (m, h) => whole(Some(m.unwrap_or(0.0) + h.unwrap_or(0.0))),
            },
        })
        .collect()
}

pub fn workout_rows(docs: &[Workout]) -> Vec<WorkoutRow> {
    docs.iter()
        .map(|w| WorkoutRow {
            date: w.day.to_string(),
            external_id: w.id.clone(),
            start_at: utc_text(w.start_datetime),
            end_at: utc_text(w.end_datetime),
            category_label: w.activity.clone(),
            duration: (w.end_datetime - w.start_datetime).num_seconds().max(0),
            distance: w.distance,
            calories: w.calories,
        })
        .collect()
}

fn observation(metric: &str, unit: &str, day: NaiveDate, doc_id: &str, value: f64) -> Observation {
    Observation {
        metric: metric.to_string(),
        measured_at: noon(day),
        value,
        unit: unit.to_string(),
        source: SourceId::OURA,
        external_id: Some(format!("{metric}:{doc_id}")),
        device: None,
    }
}

pub fn sleep_observations(rows: &[SleepRow]) -> Vec<Observation> {
    rows.iter()
        .filter_map(|r| {
            let hrv = r.average_hrv?;
            let day = r.date.parse::<NaiveDate>().ok()?;
            Some(observation("hrv", "ms", day, &r.external_id, hrv))
        })
        .collect()
}

pub fn readiness_observations(docs: &[DailyReadiness]) -> Vec<Observation> {
    docs.iter()
        .flat_map(|d| {
            [
                d.score
                    .map(|v| observation("readiness_score", "pts", d.day, &d.id, v)),
                d.temperature_deviation
                    .map(|v| observation("temperature_deviation", "degC", d.day, &d.id, v)),
            ]
        })
        .flatten()
        .collect()
}

pub fn spo2_observations(docs: &[DailySpo2]) -> Vec<Observation> {
    docs.iter()
        .filter_map(|d| {
            let average = d.spo2_percentage.as_ref()?.average?;
            Some(observation("spo2", "%", d.day, &d.id, average))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn periods(json: &str) -> Vec<SleepPeriod> {
        serde_json::from_str(json).expect("periodes")
    }

    fn scores(json: &str) -> Vec<DailySleep> {
        serde_json::from_str(json).expect("scores")
    }

    const NIGHT: &str = r#"[
        {"id":"s-nap","day":"2026-10-24","bedtime_start":"2026-10-24T14:00:00+02:00","bedtime_end":"2026-10-24T14:40:00+02:00","type":"late_nap","total_sleep_duration":2400},
        {"id":"s-night","day":"2026-10-24","bedtime_start":"2026-10-23T23:30:00+02:00","bedtime_end":"2026-10-24T07:10:00+02:00","type":"long_sleep",
         "total_sleep_duration":25200,"deep_sleep_duration":5400,"light_sleep_duration":13500,"rem_sleep_duration":6300,"awake_time":2400,
         "latency":600,"time_in_bed":27600,"efficiency":91,"average_heart_rate":54.4,"lowest_heart_rate":48,"average_hrv":62,"average_breath":14.6},
        {"id":"s-gone","day":"2026-10-24","bedtime_start":"2026-10-24T01:00:00+02:00","bedtime_end":"2026-10-24T02:00:00+02:00","type":"deleted","total_sleep_duration":3600}
    ]"#;

    #[test]
    fn the_long_sleep_of_the_day_becomes_the_row_and_naps_and_deleted_periods_are_ignored() {
        let rows = sleep_rows(&periods(NIGHT), &[]);
        assert_eq!(rows.len(), 1);
        let row = &rows[0];
        assert_eq!(row.date, "2026-10-24");
        assert_eq!(row.external_id, "s-night");
        assert_eq!(row.start_at.as_deref(), Some("2026-10-23T21:30:00Z"));
        assert_eq!(row.end_at.as_deref(), Some("2026-10-24T05:10:00Z"));
        assert_eq!(row.duration_total, Some(25200));
        assert_eq!(row.duration_deep, Some(5400));
        assert_eq!(row.duration_light, Some(13500));
        assert_eq!(row.duration_rem, Some(6300));
        assert_eq!(row.duration_awake, Some(2400));
        assert_eq!(row.duration_to_sleep, Some(600));
        assert_eq!(row.total_timeinbed, Some(27600));
        assert_eq!(row.sleep_efficiency, Some(91.0));
        assert_eq!(row.hr_average, Some(54));
        assert_eq!(row.hr_min, Some(48));
        assert_eq!(row.rr_average, Some(14.6));
        assert_eq!(row.average_hrv, Some(62.0));
        assert_eq!(row.sleep_score, None);
    }

    #[test]
    fn two_long_sleeps_on_one_day_keep_the_longest() {
        let rows = sleep_rows(
            &periods(
                r#"[
                {"id":"a","day":"2026-10-24","bedtime_start":"2026-10-24T00:00:00+02:00","bedtime_end":"2026-10-24T03:00:00+02:00","type":"long_sleep","total_sleep_duration":9000},
                {"id":"b","day":"2026-10-24","bedtime_start":"2026-10-24T04:00:00+02:00","bedtime_end":"2026-10-24T09:00:00+02:00","type":"long_sleep","total_sleep_duration":16000}]"#,
            ),
            &[],
        );
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].external_id, "b");
    }

    #[test]
    fn the_daily_score_joins_the_night_and_a_scored_day_without_a_night_still_gets_a_row() {
        let rows = sleep_rows(
            &periods(NIGHT),
            &scores(
                r#"[{"id":"d-1","day":"2026-10-24","score":83},{"id":"d-2","day":"2026-10-25","score":70},{"id":"d-3","day":"2026-10-26","score":null}]"#,
            ),
        );
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].date, "2026-10-24");
        assert_eq!(rows[0].external_id, "s-night");
        assert_eq!(rows[0].sleep_score, Some(83));
        assert_eq!(rows[1].date, "2026-10-25");
        assert_eq!(rows[1].external_id, "d-2");
        assert_eq!(rows[1].sleep_score, Some(70));
        assert_eq!(rows[1].duration_total, None);
    }

    #[test]
    fn sleep_observations_carry_the_hrv_at_noon_utc_of_the_oura_day() {
        let rows = sleep_rows(&periods(NIGHT), &[]);
        let obs = sleep_observations(&rows);
        assert_eq!(obs.len(), 1);
        assert_eq!(obs[0].metric, "hrv");
        assert_eq!(obs[0].value, 62.0);
        assert_eq!(obs[0].unit, "ms");
        assert_eq!(obs[0].source, SourceId::OURA);
        assert_eq!(obs[0].external_id.as_deref(), Some("hrv:s-night"));
        assert_eq!(
            obs[0].measured_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            "2026-10-24T12:00:00Z"
        );
    }

    #[test]
    fn a_daily_activity_maps_to_one_row_with_the_active_time_as_moderate_plus_intense() {
        let docs: Vec<DailyActivity> = serde_json::from_str(
            r#"[{"id":"a-1","day":"2026-10-24","steps":9876,"active_calories":412,"total_calories":2410,
                 "equivalent_walking_distance":7800,"low_activity_time":5400,"medium_activity_time":1800,"high_activity_time":600}]"#,
        )
        .expect("activite");
        let rows = activity_rows(&docs);
        assert_eq!(rows.len(), 1);
        let row = &rows[0];
        assert_eq!(row.date, "2026-10-24");
        assert_eq!(row.external_id, "a-1");
        assert_eq!(row.steps, Some(9876));
        assert_eq!(row.distance, Some(7800.0));
        assert_eq!(row.calories_active, Some(412.0));
        assert_eq!(row.calories_total, Some(2410.0));
        assert_eq!(row.duration_soft, Some(5400));
        assert_eq!(row.duration_moderate, Some(1800));
        assert_eq!(row.duration_intense, Some(600));
        assert_eq!(row.duration_active, Some(2400));
    }

    #[test]
    fn a_workout_maps_to_its_label_window_and_duration_in_seconds() {
        let docs: Vec<Workout> = serde_json::from_str(
            r#"[{"id":"w-1","activity":"running","day":"2026-10-24","start_datetime":"2026-10-24T18:00:00+02:00",
                 "end_datetime":"2026-10-24T18:45:30+02:00","calories":512.5,"distance":8200.0,"label":null}]"#,
        )
        .expect("seance");
        let rows = workout_rows(&docs);
        assert_eq!(rows.len(), 1);
        let row = &rows[0];
        assert_eq!(row.external_id, "w-1");
        assert_eq!(row.date, "2026-10-24");
        assert_eq!(row.category_label, "running");
        assert_eq!(row.start_at, "2026-10-24T16:00:00Z");
        assert_eq!(row.end_at, "2026-10-24T16:45:30Z");
        assert_eq!(row.duration, 2730);
        assert_eq!(row.distance, Some(8200.0));
        assert_eq!(row.calories, Some(512.5));
    }

    #[test]
    fn readiness_gives_two_observations_with_distinct_external_ids() {
        let docs: Vec<DailyReadiness> = serde_json::from_str(
            r#"[{"id":"r-1","day":"2026-10-24","score":78,"temperature_deviation":-0.23},
                {"id":"r-2","day":"2026-10-25","score":null,"temperature_deviation":null}]"#,
        )
        .expect("readiness");
        let obs = readiness_observations(&docs);
        let ids: Vec<(&str, f64, &str)> = obs
            .iter()
            .map(|o| {
                (
                    o.metric.as_str(),
                    o.value,
                    o.external_id.as_deref().unwrap(),
                )
            })
            .collect();
        assert_eq!(
            ids,
            vec![
                ("readiness_score", 78.0, "readiness_score:r-1"),
                ("temperature_deviation", -0.23, "temperature_deviation:r-1"),
            ]
        );
    }

    #[test]
    fn spo2_keeps_the_nightly_average_and_skips_days_without_one() {
        let docs: Vec<DailySpo2> = serde_json::from_str(
            r#"[{"id":"o-1","day":"2026-10-24","spo2_percentage":{"average":97.4}},
                {"id":"o-2","day":"2026-10-25","spo2_percentage":null}]"#,
        )
        .expect("spo2");
        let obs = spo2_observations(&docs);
        assert_eq!(obs.len(), 1);
        assert_eq!(obs[0].metric, "spo2");
        assert_eq!(obs[0].value, 97.4);
        assert_eq!(obs[0].unit, "%");
        assert_eq!(obs[0].external_id.as_deref(), Some("spo2:o-1"));
    }
}
