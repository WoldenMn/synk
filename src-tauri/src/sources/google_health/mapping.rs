use std::collections::BTreeMap;

use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};

use super::types::{
    number, ExercisePoint, Flex, HrvPoint, Interval, RespiratoryPoint, RestingHeartRatePoint,
    Rollup, SleepPoint, Spo2Point, TemperaturePoint,
};
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
    pub sleep_efficiency: Option<f64>,
    pub rr_average: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ActivityRow {
    pub date: String,
    pub external_id: String,
    pub steps: Option<i64>,
    pub distance: Option<f64>,
    pub calories_active: Option<f64>,
    pub calories_total: Option<f64>,
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

fn utc_text(at: DateTime<Utc>) -> String {
    at.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn offset_seconds(offset: &Option<String>) -> i64 {
    offset
        .as_deref()
        .and_then(|raw| raw.strip_suffix('s'))
        .and_then(|digits| digits.parse().ok())
        .unwrap_or(0)
}

fn local_date(at: DateTime<Utc>, offset: &Option<String>) -> NaiveDate {
    (at + Duration::seconds(offset_seconds(offset))).date_naive()
}

fn noon(day: NaiveDate) -> DateTime<Utc> {
    Utc.from_utc_datetime(&day.and_hms_opt(12, 0, 0).unwrap_or_default())
}

fn last_segment(name: &Option<String>) -> Option<String> {
    name.as_deref()
        .and_then(|n| n.rsplit('/').next())
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn seconds_of_minutes(value: &Option<Flex>) -> Option<i64> {
    number(value).map(|m| (m * 60.0).round() as i64)
}

fn stage_seconds(point: &SleepPoint, kind: &str) -> Option<i64> {
    let summary = point.sleep.summary.as_ref()?;
    summary
        .stages_summary
        .iter()
        .find(|s| s.kind == kind)
        .and_then(|s| seconds_of_minutes(&s.minutes))
}

fn night_row(point: &SleepPoint) -> SleepRow {
    let interval = &point.sleep.interval;
    let day = local_date(interval.end_time, &interval.end_utc_offset);
    let summary = point.sleep.summary.as_ref();
    let asleep = summary.and_then(|s| number(&s.minutes_asleep));
    let in_bed = summary.and_then(|s| number(&s.minutes_in_sleep_period));
    SleepRow {
        date: day.to_string(),
        external_id: last_segment(&point.name).unwrap_or_else(|| format!("sleep:{day}")),
        start_at: Some(utc_text(interval.start_time)),
        end_at: Some(utc_text(interval.end_time)),
        duration_total: summary.and_then(|s| seconds_of_minutes(&s.minutes_asleep)),
        duration_deep: stage_seconds(point, "DEEP"),
        duration_light: stage_seconds(point, "LIGHT"),
        duration_rem: stage_seconds(point, "REM"),
        duration_awake: summary.and_then(|s| seconds_of_minutes(&s.minutes_awake)),
        duration_to_sleep: summary.and_then(|s| seconds_of_minutes(&s.minutes_to_fall_asleep)),
        total_timeinbed: summary.and_then(|s| seconds_of_minutes(&s.minutes_in_sleep_period)),
        sleep_efficiency: match (asleep, in_bed) {
            (Some(a), Some(b)) if b > 0.0 => Some(a / b * 100.0),
            _ => None,
        },
        rr_average: None,
    }
}

fn asleep_minutes(point: &SleepPoint) -> f64 {
    point
        .sleep
        .summary
        .as_ref()
        .and_then(|s| number(&s.minutes_asleep))
        .unwrap_or(0.0)
}

fn is_main(point: &SleepPoint) -> bool {
    point
        .sleep
        .metadata
        .as_ref()
        .and_then(|m| m.main)
        .unwrap_or(true)
}

pub fn sleep_rows(points: &[SleepPoint], breathing: &[RespiratoryPoint]) -> Vec<SleepRow> {
    let mut best: BTreeMap<NaiveDate, &SleepPoint> = BTreeMap::new();
    for point in points.iter().filter(|p| is_main(p)) {
        let interval = &point.sleep.interval;
        let day = local_date(interval.end_time, &interval.end_utc_offset);
        let better = best
            .get(&day)
            .is_none_or(|kept| asleep_minutes(point) > asleep_minutes(kept));
        if better {
            best.insert(day, point);
        }
    }
    let rates: BTreeMap<NaiveDate, f64> = breathing
        .iter()
        .filter_map(|p| {
            let r = &p.daily_respiratory_rate;
            Some((r.date.date()?, number(&r.breaths_per_minute)?))
        })
        .collect();
    best.into_iter()
        .map(|(day, point)| {
            let mut row = night_row(point);
            row.rr_average = rates.get(&day).copied();
            row
        })
        .collect()
}

pub fn activity_rows(rollups: &[Rollup]) -> Vec<ActivityRow> {
    let mut rows: BTreeMap<NaiveDate, ActivityRow> = BTreeMap::new();
    for rollup in rollups {
        let Some(day) = rollup.civil_start_time.date.date() else {
            continue;
        };
        let row = rows.entry(day).or_insert_with(|| ActivityRow {
            date: day.to_string(),
            external_id: format!("activity:{day}"),
            ..ActivityRow::default()
        });
        if let Some(steps) = rollup.steps.as_ref().and_then(|s| number(&s.count_sum)) {
            row.steps = Some(steps.round() as i64);
        }
        if let Some(mm) = rollup
            .distance
            .as_ref()
            .and_then(|d| number(&d.millimeters_sum))
        {
            row.distance = Some(mm / 1000.0);
        }
        if let Some(kcal) = rollup
            .active_energy_burned
            .as_ref()
            .and_then(|e| number(&e.kcal_sum))
        {
            row.calories_active = Some(kcal);
        }
        if let Some(kcal) = rollup
            .total_calories
            .as_ref()
            .and_then(|e| number(&e.kcal_sum))
        {
            row.calories_total = Some(kcal);
        }
    }
    rows.into_values().collect()
}

fn active_seconds(raw: &Option<String>) -> Option<i64> {
    let digits = raw.as_deref()?.strip_suffix('s')?;
    digits.parse::<f64>().ok().map(|s| s.round() as i64)
}

fn span(interval: &Interval) -> i64 {
    (interval.end_time - interval.start_time)
        .num_seconds()
        .max(0)
}

pub fn workout_rows(points: &[ExercisePoint]) -> Vec<WorkoutRow> {
    points
        .iter()
        .map(|p| {
            let e = &p.exercise;
            let start = e.interval.start_time;
            let label = e
                .display_name
                .as_deref()
                .filter(|n| !n.is_empty())
                .or(e.exercise_type.as_deref())
                .unwrap_or("exercise")
                .to_lowercase();
            let metrics = e.metrics_summary.as_ref();
            WorkoutRow {
                date: local_date(start, &e.interval.start_utc_offset).to_string(),
                external_id: last_segment(&p.name)
                    .unwrap_or_else(|| format!("exercise:{}", utc_text(start))),
                start_at: utc_text(start),
                end_at: utc_text(e.interval.end_time),
                category_label: label,
                duration: active_seconds(&e.active_duration).unwrap_or_else(|| span(&e.interval)),
                distance: metrics
                    .and_then(|m| number(&m.distance_millimeters))
                    .map(|mm| mm / 1000.0),
                calories: metrics.and_then(|m| number(&m.calories_kcal)),
            }
        })
        .collect()
}

fn observation(metric: &str, unit: &str, day: NaiveDate, value: f64) -> Observation {
    Observation {
        metric: metric.to_string(),
        measured_at: noon(day),
        value,
        unit: unit.to_string(),
        source: SourceId::GOOGLE_HEALTH,
        external_id: Some(format!("{metric}:{day}")),
        device: None,
    }
}

pub fn daily_observations(
    resting: &[RestingHeartRatePoint],
    hrv: &[HrvPoint],
    spo2: &[Spo2Point],
    temperature: &[TemperaturePoint],
) -> Vec<Observation> {
    let mut out = Vec::new();
    for p in resting {
        let r = &p.daily_resting_heart_rate;
        if let (Some(day), Some(v)) = (r.date.date(), number(&r.beats_per_minute)) {
            out.push(observation("resting_heart_rate", "bpm", day, v));
        }
    }
    for p in hrv {
        let h = &p.daily_heart_rate_variability;
        if let (Some(day), Some(v)) = (
            h.date.date(),
            number(&h.average_heart_rate_variability_milliseconds),
        ) {
            out.push(observation("hrv", "ms", day, v));
        }
    }
    for p in spo2 {
        let s = &p.daily_oxygen_saturation;
        if let (Some(day), Some(v)) = (s.date.date(), number(&s.average_percentage)) {
            out.push(observation("spo2", "%", day, v));
        }
    }
    for p in temperature {
        let t = &p.daily_sleep_temperature_derivations;
        if let (Some(day), Some(night), Some(base)) = (
            t.date.date(),
            number(&t.nightly_temperature_celsius),
            number(&t.baseline_temperature_celsius),
        ) {
            out.push(observation(
                "temperature_deviation",
                "degC",
                day,
                night - base,
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sleeps(json: &str) -> Vec<SleepPoint> {
        serde_json::from_str(json).expect("nuits")
    }

    const NIGHTS: &str = r#"[
      {"name":"users/1/dataTypes/sleep/dataPoints/nap","sleep":{"interval":{"startTime":"2026-10-24T13:00:00Z","startUtcOffset":"7200s","endTime":"2026-10-24T13:40:00Z","endUtcOffset":"7200s"},
        "metadata":{"main":false},"summary":{"minutesAsleep":"35"}}},
      {"name":"users/1/dataTypes/sleep/dataPoints/night","sleep":{"interval":{"startTime":"2026-10-23T21:30:00Z","startUtcOffset":"7200s","endTime":"2026-10-24T05:10:00Z","endUtcOffset":"7200s"},
        "type":"STAGES","metadata":{"stagesStatus":"SUCCEEDED","processed":true,"main":true},
        "summary":{"minutesInSleepPeriod":"460","minutesAfterWakeUp":"0","minutesToFallAsleep":"10","minutesAsleep":"420","minutesAwake":"40",
          "stagesSummary":[{"type":"AWAKE","minutes":"40","count":"9"},{"type":"LIGHT","minutes":"225","count":"19"},{"type":"DEEP","minutes":"90","count":"6"},{"type":"REM","minutes":"105","count":"4"}]}}},
      {"name":"users/1/dataTypes/sleep/dataPoints/old","sleep":{"interval":{"startTime":"2026-10-23T00:00:00Z","endTime":"2026-10-23T06:00:00Z"},"metadata":{"mainSleep":true},"summary":{"minutesAsleep":"300","minutesInSleepPeriod":"360"}}}
    ]"#;

    #[test]
    fn the_main_night_becomes_the_row_dated_by_its_local_end_and_naps_are_ignored() {
        let rows = sleep_rows(&sleeps(NIGHTS), &[]);
        assert_eq!(rows.len(), 2);
        let night = rows.iter().find(|r| r.date == "2026-10-24").expect("nuit");
        assert_eq!(night.external_id, "night");
        assert_eq!(night.start_at.as_deref(), Some("2026-10-23T21:30:00Z"));
        assert_eq!(night.end_at.as_deref(), Some("2026-10-24T05:10:00Z"));
        assert_eq!(night.duration_total, Some(25_200));
        assert_eq!(night.duration_awake, Some(2_400));
        assert_eq!(night.duration_to_sleep, Some(600));
        assert_eq!(night.total_timeinbed, Some(27_600));
        assert_eq!(night.duration_light, Some(13_500));
        assert_eq!(night.duration_deep, Some(5_400));
        assert_eq!(night.duration_rem, Some(6_300));
        let efficiency = night.sleep_efficiency.expect("efficacite");
        assert!((efficiency - 91.3).abs() < 0.1, "{efficiency}");
    }

    #[test]
    fn a_night_ending_after_local_midnight_keeps_the_local_date_not_the_utc_one() {
        let late = r#"[{"name":"users/1/dataTypes/sleep/dataPoints/late","sleep":{"interval":{"startTime":"2026-10-24T20:00:00Z","startUtcOffset":"-18000s","endTime":"2026-10-25T03:00:00Z","endUtcOffset":"-18000s"},"metadata":{"main":true},"summary":{"minutesAsleep":"400"}}}]"#;
        let rows = sleep_rows(&sleeps(late), &[]);
        assert_eq!(rows[0].date, "2026-10-24");
    }

    #[test]
    fn a_missing_stage_breakdown_and_the_alias_spelling_are_both_tolerated() {
        let rows = sleep_rows(&sleeps(NIGHTS), &[]);
        let old = rows
            .iter()
            .find(|r| r.date == "2026-10-23")
            .expect("ancienne");
        assert_eq!(old.duration_total, Some(18_000));
        assert_eq!(old.duration_deep, None);
        assert_eq!(old.duration_rem, None);
    }

    #[test]
    fn the_longest_main_sleep_of_a_day_wins_and_breathing_is_joined_by_date() {
        let two = r#"[
          {"name":"a","sleep":{"interval":{"startTime":"2026-10-23T22:00:00Z","endTime":"2026-10-24T01:00:00Z"},"metadata":{"main":true},"summary":{"minutesAsleep":"150"}}},
          {"name":"b","sleep":{"interval":{"startTime":"2026-10-24T01:30:00Z","endTime":"2026-10-24T08:00:00Z"},"metadata":{"main":true},"summary":{"minutesAsleep":"380"}}}]"#;
        let breathing: Vec<RespiratoryPoint> = serde_json::from_str(
            r#"[{"dailyRespiratoryRate":{"date":{"year":2026,"month":10,"day":24},"breathsPerMinute":14.6}}]"#,
        )
        .unwrap();
        let rows = sleep_rows(&sleeps(two), &breathing);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].external_id, "b");
        assert_eq!(rows[0].rr_average, Some(14.6));
    }

    fn rollups(json: &str) -> Vec<Rollup> {
        serde_json::from_str(json).expect("cumuls")
    }

    #[test]
    fn rollups_of_four_kinds_merge_into_one_activity_row_per_day_in_metres() {
        let day = r#""civilStartTime":{"date":{"year":2026,"month":10,"day":25}}"#;
        let json = format!(
            r#"[{{{day},"steps":{{"countSum":"9876"}}}},
               {{{day},"distance":{{"millimetersSum":"7800000"}}}},
               {{{day},"activeEnergyBurned":{{"kcalSum":512.5}}}},
               {{{day},"totalCalories":{{"kcalSum":2450.2}}}},
               {{"civilStartTime":{{"date":{{"year":2026,"month":10,"day":24}}}},"steps":{{"countSum":"0"}}}}]"#
        );
        let rows = activity_rows(&rollups(&json));
        assert_eq!(rows.len(), 2);
        let busy = rows.iter().find(|r| r.date == "2026-10-25").unwrap();
        assert_eq!(busy.steps, Some(9876));
        assert_eq!(busy.distance, Some(7800.0));
        assert_eq!(busy.calories_active, Some(512.5));
        assert_eq!(busy.calories_total, Some(2450.2));
        let idle = rows.iter().find(|r| r.date == "2026-10-24").unwrap();
        assert_eq!(idle.steps, Some(0));
        assert_eq!(idle.distance, None);
    }

    #[test]
    fn a_session_becomes_a_workout_with_its_local_date_and_both_distance_spellings() {
        let json = r#"[
          {"name":"users/1/dataTypes/exercise/dataPoints/777","exercise":{"interval":{"startTime":"2026-10-24T14:00:00Z","startUtcOffset":"7200s","endTime":"2026-10-24T14:45:30Z","endUtcOffset":"7200s"},
            "exerciseType":"RUNNING","displayName":"Run","activeDuration":"2700s","metricsSummary":{"caloriesKcal":410.5,"distanceMillimeters":8200000}}},
          {"name":"users/1/dataTypes/exercise/dataPoints/778","exercise":{"interval":{"startTime":"2026-10-25T08:00:00Z","endTime":"2026-10-25T08:20:00Z"},
            "exerciseType":"WALKING","metricsSummary":{"distanceMillimiters":1609344}}}]"#;
        let points: Vec<ExercisePoint> = serde_json::from_str(json).unwrap();
        let rows = workout_rows(&points);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].external_id, "777");
        assert_eq!(rows[0].date, "2026-10-24");
        assert_eq!(rows[0].category_label, "run");
        assert_eq!(rows[0].duration, 2_700);
        assert_eq!(rows[0].distance, Some(8200.0));
        assert_eq!(rows[0].calories, Some(410.5));
        assert_eq!(rows[0].start_at, "2026-10-24T14:00:00Z");
        assert_eq!(rows[1].category_label, "walking");
        assert_eq!(rows[1].duration, 1_200);
        assert_eq!(rows[1].distance, Some(1609.344));
    }

    #[test]
    fn daily_summaries_become_observations_at_noon_utc_with_a_stable_id() {
        let d = r#""date":{"year":2026,"month":10,"day":25}"#;
        let resting: Vec<RestingHeartRatePoint> = serde_json::from_str(&format!(
            r#"[{{"dailyRestingHeartRate":{{{d},"beatsPerMinute":"52"}}}}]"#
        ))
        .unwrap();
        let hrv: Vec<HrvPoint> = serde_json::from_str(&format!(
            r#"[{{"dailyHeartRateVariability":{{{d},"averageHeartRateVariabilityMilliseconds":48.2}}}}]"#
        ))
        .unwrap();
        let spo2: Vec<Spo2Point> = serde_json::from_str(&format!(
            r#"[{{"dailyOxygenSaturation":{{{d},"averagePercentage":96.4}}}}]"#
        ))
        .unwrap();
        let temperature: Vec<TemperaturePoint> = serde_json::from_str(&format!(
            r#"[{{"dailySleepTemperatureDerivations":{{{d},"nightlyTemperatureCelsius":33.1,"baselineTemperatureCelsius":32.6}}}},
               {{"dailySleepTemperatureDerivations":{{"date":{{"year":2026,"month":10,"day":24}},"nightlyTemperatureCelsius":33.0}}}}]"#
        ))
        .unwrap();

        let all = daily_observations(&resting, &hrv, &spo2, &temperature);

        assert_eq!(all.len(), 4);
        let get = |m: &str| all.iter().find(|o| o.metric == m).expect(m);
        assert_eq!(get("resting_heart_rate").value, 52.0);
        assert_eq!(get("resting_heart_rate").unit, "bpm");
        assert_eq!(get("hrv").value, 48.2);
        assert_eq!(get("spo2").value, 96.4);
        let deviation = get("temperature_deviation");
        assert!((deviation.value - 0.5).abs() < 1e-9, "{}", deviation.value);
        assert_eq!(deviation.unit, "degC");
        assert_eq!(
            deviation.measured_at.to_rfc3339(),
            "2026-10-25T12:00:00+00:00"
        );
        assert_eq!(
            deviation.external_id.as_deref(),
            Some("temperature_deviation:2026-10-25")
        );
        assert_eq!(deviation.source, SourceId::GOOGLE_HEALTH);
    }
}
