use std::collections::BTreeMap;

use chrono::{DateTime, Utc};

use crate::sources::Observation;

#[derive(Debug, Clone, PartialEq)]
pub struct BpRow {
    pub measured_at: DateTime<Utc>,
    pub systolic: i64,
    pub diastolic: i64,
    pub external_id: String,
    pub device: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SleepRow {
    pub date: String,
    pub external_id: String,
    pub start_at: String,
    pub end_at: String,
    pub duration_total: i64,
    pub duration_deep: i64,
    pub duration_light: i64,
    pub duration_rem: i64,
    pub duration_awake: i64,
    pub total_timeinbed: Option<i64>,
    pub device: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ActivityRow {
    pub date: String,
    pub external_id: String,
    pub steps: Option<i64>,
    pub distance: Option<f64>,
    pub calories_active: Option<f64>,
    pub calories_total: Option<f64>,
    pub device: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct WorkoutRow {
    pub date: String,
    pub external_id: String,
    pub start_at: String,
    pub end_at: String,
    pub category_label: String,
    pub duration: i64,
    pub distance: Option<f64>,
    pub calories: Option<f64>,
    pub elevation: Option<f64>,
    pub hr_average: Option<i64>,
    pub hr_min: Option<i64>,
    pub hr_max: Option<i64>,
    pub device: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Out {
    Observation(Observation),
    BloodPressure(BpRow),
    Sleep(SleepRow),
    Activity(ActivityRow),
    Workout(WorkoutRow),
}

impl Out {
    pub fn with_bounded_device(mut self) -> Self {
        let cut = |d: &mut String| *d = bounded(d, MAX_DEVICE_CHARS);
        match &mut self {
            Out::Observation(o) => {
                if let Some(d) = o.device.as_mut() {
                    cut(d);
                }
            }
            Out::BloodPressure(r) => cut(&mut r.device),
            Out::Sleep(r) => cut(&mut r.device),
            Out::Activity(r) => cut(&mut r.device),
            Out::Workout(r) => cut(&mut r.device),
        }
        self
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Skipped(pub BTreeMap<String, u64>);

pub const MAX_REASONS: usize = 50;
pub const MAX_REASON_CHARS: usize = 120;
pub const MAX_DEVICE_CHARS: usize = 80;
pub const OTHER_REASONS: &str = "autres motifs";

pub fn bounded(text: &str, max: usize) -> String {
    text.chars().take(max).collect()
}

impl Skipped {
    pub fn add(&mut self, reason: impl Into<String>) {
        self.add_many(reason, 1);
    }

    pub fn add_many(&mut self, reason: impl Into<String>, count: u64) {
        let reason = bounded(&reason.into(), MAX_REASON_CHARS);
        let key = if self.0.contains_key(&reason) || self.0.len() < MAX_REASONS {
            reason
        } else {
            OTHER_REASONS.to_string()
        };
        *self.0.entry(key).or_insert(0) += count;
    }

    pub fn total(&self) -> u64 {
        self.0.values().sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ten_thousand_distinct_reasons_stay_bounded_and_keep_their_count() {
        let mut skipped = Skipped::default();
        for i in 0..10_000 {
            skipped.add(format!("unité inconnue : u{i}"));
        }
        assert!(skipped.0.len() <= MAX_REASONS + 1, "{}", skipped.0.len());
        assert_eq!(skipped.total(), 10_000);
        assert!(skipped.0.contains_key(OTHER_REASONS));
        skipped.add("unité inconnue : u0");
        assert_eq!(skipped.0["unité inconnue : u0"], 2);
    }

    #[test]
    fn a_huge_device_name_is_cut_on_every_row_kind() {
        let long = "d".repeat(5_000);
        let out = Out::Sleep(SleepRow {
            device: long.clone(),
            ..SleepRow::default()
        })
        .with_bounded_device();
        let Out::Sleep(r) = out else { panic!() };
        assert_eq!(r.device.chars().count(), MAX_DEVICE_CHARS);
    }

    #[test]
    fn a_huge_reason_is_cut() {
        let mut skipped = Skipped::default();
        skipped.add("x".repeat(100_000));
        assert!(skipped
            .0
            .keys()
            .all(|k| k.chars().count() <= MAX_REASON_CHARS));
    }
}
