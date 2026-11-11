use std::collections::BTreeMap;

use chrono::{DateTime, Duration, NaiveDateTime, Timelike, Utc};

use super::rows::SleepRow;
use super::time::{utc_text, Stamp};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stage {
    InBed,
    Awake,
    Core,
    Deep,
    Rem,
    Unspecified,
}

pub fn stage_of(value: &str) -> Option<Stage> {
    match value.strip_prefix("HKCategoryValueSleepAnalysis")? {
        "InBed" => Some(Stage::InBed),
        "Awake" => Some(Stage::Awake),
        "AsleepCore" => Some(Stage::Core),
        "AsleepDeep" => Some(Stage::Deep),
        "AsleepREM" => Some(Stage::Rem),
        "Asleep" | "AsleepUnspecified" => Some(Stage::Unspecified),
        _ => None,
    }
}

#[derive(Default)]
struct Night {
    deep: f64,
    core: f64,
    rem: f64,
    unspecified: f64,
    awake: f64,
    in_bed: f64,
    start: Option<DateTime<Utc>>,
    end: Option<DateTime<Utc>>,
}

impl Night {
    fn asleep(&self) -> f64 {
        self.deep + self.core + self.rem + self.unspecified
    }
}

#[derive(Default)]
pub struct SleepAgg {
    nights: BTreeMap<String, BTreeMap<String, Night>>,
}

fn night_date(local: NaiveDateTime) -> String {
    (local + Duration::hours(6)).format("%Y-%m-%d").to_string()
}

fn is_nap(local: NaiveDateTime) -> bool {
    (12..18).contains(&local.hour())
}

impl SleepAgg {
    pub fn add(&mut self, origin: &str, stage: Stage, start: &Stamp, end: &Stamp) -> bool {
        if is_nap(start.local) {
            return false;
        }
        let seconds = (end.utc - start.utc).num_seconds().max(0) as f64;
        let night = self
            .nights
            .entry(night_date(start.local))
            .or_default()
            .entry(origin.to_string())
            .or_default();
        match stage {
            Stage::InBed => night.in_bed += seconds,
            Stage::Awake => night.awake += seconds,
            Stage::Core => night.core += seconds,
            Stage::Deep => night.deep += seconds,
            Stage::Rem => night.rem += seconds,
            Stage::Unspecified => night.unspecified += seconds,
        }
        night.start = Some(night.start.map_or(start.utc, |s| s.min(start.utc)));
        night.end = Some(night.end.map_or(end.utc, |e| e.max(end.utc)));
        true
    }

    pub fn rows(self, prefix: &str) -> (Vec<SleepRow>, u64) {
        let mut rows = Vec::new();
        let mut without_sleep = 0;
        for (date, origins) in self.nights {
            let best = origins
                .into_iter()
                .max_by(|a, b| a.1.asleep().total_cmp(&b.1.asleep()));
            match best {
                Some((origin, n)) if n.asleep() > 0.0 => {
                    rows.push(row_of(prefix, date, origin, &n))
                }
                _ => without_sleep += 1,
            }
        }
        (rows, without_sleep)
    }
}

fn row_of(prefix: &str, date: String, origin: String, n: &Night) -> SleepRow {
    let span = match (n.start, n.end) {
        (Some(s), Some(e)) => (e - s).num_seconds().max(0),
        _ => 0,
    };
    SleepRow {
        external_id: format!("{prefix}:sleep:{date}"),
        start_at: n.start.map(utc_text).unwrap_or_default(),
        end_at: n.end.map(utc_text).unwrap_or_default(),
        duration_total: n.asleep().round() as i64,
        duration_deep: n.deep.round() as i64,
        duration_light: n.core.round() as i64,
        duration_rem: n.rem.round() as i64,
        duration_awake: n.awake.round() as i64,
        total_timeinbed: Some(if n.in_bed > 0.0 {
            n.in_bed.round() as i64
        } else {
            span
        }),
        device: origin,
        date,
    }
}

#[cfg(test)]
mod tests {
    use super::super::apple_health::dates::parse;
    use super::*;

    fn seg(agg: &mut SleepAgg, origin: &str, stage: Stage, a: &str, b: &str) -> bool {
        agg.add(origin, stage, &parse(a).unwrap(), &parse(b).unwrap())
    }

    #[test]
    fn stages_are_read_from_the_healthkit_values_including_the_legacy_one() {
        assert_eq!(
            stage_of("HKCategoryValueSleepAnalysisAsleepREM"),
            Some(Stage::Rem)
        );
        assert_eq!(
            stage_of("HKCategoryValueSleepAnalysisAsleep"),
            Some(Stage::Unspecified)
        );
        assert_eq!(
            stage_of("HKCategoryValueSleepAnalysisInBed"),
            Some(Stage::InBed)
        );
        assert_eq!(stage_of("HKCategoryValueSleepAnalysisSomethingNew"), None);
        assert_eq!(stage_of("garbage"), None);
    }

    #[test]
    fn a_night_across_midnight_lands_on_the_wake_up_date_with_its_stages() {
        let mut agg = SleepAgg::default();
        let z = "+0200";
        seg(
            &mut agg,
            "Watch",
            Stage::InBed,
            &format!("2026-10-25 23:20:00 {z}"),
            &format!("2026-10-26 07:10:00 {z}"),
        );
        seg(
            &mut agg,
            "Watch",
            Stage::Core,
            &format!("2026-10-25 23:30:00 {z}"),
            &format!("2026-10-26 02:30:00 {z}"),
        );
        seg(
            &mut agg,
            "Watch",
            Stage::Deep,
            &format!("2026-10-26 02:30:00 {z}"),
            &format!("2026-10-26 04:00:00 {z}"),
        );
        seg(
            &mut agg,
            "Watch",
            Stage::Rem,
            &format!("2026-10-26 04:00:00 {z}"),
            &format!("2026-10-26 05:00:00 {z}"),
        );
        seg(
            &mut agg,
            "Watch",
            Stage::Awake,
            &format!("2026-10-26 05:00:00 {z}"),
            &format!("2026-10-26 05:20:00 {z}"),
        );
        let (rows, empty) = agg.rows("ah");
        assert_eq!(empty, 0);
        assert_eq!(rows.len(), 1);
        let r = &rows[0];
        assert_eq!(r.date, "2026-10-26");
        assert_eq!(r.duration_light, 10_800);
        assert_eq!(r.duration_deep, 5_400);
        assert_eq!(r.duration_rem, 3_600);
        assert_eq!(r.duration_total, 19_800);
        assert_eq!(r.duration_awake, 1_200);
        assert_eq!(r.total_timeinbed, Some(28_200));
        assert_eq!(r.start_at, "2026-10-25T21:20:00Z");
        assert_eq!(r.end_at, "2026-10-26T05:10:00Z");
        assert_eq!(r.external_id, "ah:sleep:2026-10-26");
    }

    #[test]
    fn an_afternoon_nap_is_left_out_and_the_fullest_origin_wins_the_night() {
        let mut agg = SleepAgg::default();
        assert!(!seg(
            &mut agg,
            "Watch",
            Stage::Unspecified,
            "2026-10-26 14:00:00 +0200",
            "2026-10-26 14:40:00 +0200"
        ));
        seg(
            &mut agg,
            "Phone",
            Stage::Unspecified,
            "2026-10-25 23:30:00 +0200",
            "2026-10-26 03:30:00 +0200",
        );
        seg(
            &mut agg,
            "Watch",
            Stage::Unspecified,
            "2026-10-25 23:30:00 +0200",
            "2026-10-26 07:00:00 +0200",
        );
        let (rows, _) = agg.rows("ah");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].device, "Watch");
        assert_eq!(rows[0].duration_total, 27_000);
        assert_eq!(rows[0].total_timeinbed, Some(27_000));
    }

    #[test]
    fn a_night_with_only_in_bed_time_gives_no_row() {
        let mut agg = SleepAgg::default();
        seg(
            &mut agg,
            "Phone",
            Stage::InBed,
            "2026-10-25 23:30:00 +0200",
            "2026-10-26 07:00:00 +0200",
        );
        let (rows, empty) = agg.rows("ah");
        assert!(rows.is_empty());
        assert_eq!(empty, 1);
    }
}
