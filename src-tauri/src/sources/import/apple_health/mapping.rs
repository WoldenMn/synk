use std::collections::HashMap;

use super::activity::{ActivityAgg, Quantity};
use super::dates::{local_day, parse};
use super::items::{Item, RawRecord};
use super::units::{distance_m, energy_kcal, glucose_mmol_l, mass_kg, percent, temperature_c};
use super::workout::workout_row;
use crate::sources::import::fingerprint::record_id;
use crate::sources::import::rows::{BpRow, Out, Skipped};
use crate::sources::import::sleep::{stage_of, SleepAgg};
use crate::sources::{Observation, SourceId};

const QUANTITY_PREFIX: &str = "HKQuantityTypeIdentifier";
const SLEEP: &str = "HKCategoryTypeIdentifierSleepAnalysis";

type Convert = fn(f64, &str) -> Option<f64>;

fn same(value: f64, _unit: &str) -> Option<f64> {
    Some(value)
}

struct Metric {
    slug: &'static str,
    unit: &'static str,
    convert: Convert,
}

fn metric_of(short: &str) -> Option<Metric> {
    let (slug, unit, convert): (&'static str, &'static str, Convert) = match short {
        "BodyMass" => ("weight", "kg", mass_kg),
        "BodyFatPercentage" => ("fat_ratio", "%", percent),
        "LeanBodyMass" => ("fat_free_mass", "kg", mass_kg),
        "BodyMassIndex" => ("bmi", "kg/m2", same),
        "BodyTemperature" => ("body_temperature", "degC", temperature_c),
        "OxygenSaturation" => ("spo2", "%", percent),
        "VO2Max" => ("vo2max", "ml/kg/min", same),
        "RestingHeartRate" => ("resting_heart_rate", "bpm", same),
        "HeartRateVariabilitySDNN" => ("hrv_sdnn", "ms", same),
        "BloodGlucose" => ("blood_glucose", "mmol/L", glucose_mmol_l),
        _ => return None,
    };
    Some(Metric {
        slug,
        unit,
        convert,
    })
}

fn activity_of(short: &str) -> Option<(Quantity, Convert)> {
    Some(match short {
        "StepCount" => (Quantity::Steps, same as Convert),
        "DistanceWalkingRunning" => (Quantity::Distance, distance_m),
        "ActiveEnergyBurned" => (Quantity::Active, energy_kcal),
        "BasalEnergyBurned" => (Quantity::Basal, energy_kcal),
        _ => return None,
    })
}

pub fn wants(kind: &str) -> bool {
    if kind == SLEEP {
        return true;
    }
    let Some(short) = kind.strip_prefix(QUANTITY_PREFIX) else {
        return false;
    };
    metric_of(short).is_some()
        || activity_of(short).is_some()
        || matches!(short, "BloodPressureSystolic" | "BloodPressureDiastolic")
}

type PairKey = (i64, String);

#[derive(Default)]
pub struct Mapper {
    activity: ActivityAgg,
    sleep: SleepAgg,
    pairs: HashMap<PairKey, (Option<f64>, Option<f64>)>,
    pub skipped: Skipped,
}

impl Mapper {
    pub fn feed(&mut self, item: Item, out: &mut Vec<Out>) {
        match item {
            Item::Workout(w) => {
                if let Some(row) = workout_row(&w, &mut self.skipped) {
                    out.push(Out::Workout(row));
                }
            }
            Item::Record(r) => self.record(&r, out),
        }
    }

    fn record(&mut self, r: &RawRecord, out: &mut Vec<Out>) {
        let (Some(start), Some(end)) = (parse(&r.start), parse(&r.end)) else {
            self.skipped.add("mesure : date illisible");
            return;
        };
        if r.kind == SLEEP {
            match stage_of(&r.value) {
                Some(stage) => {
                    self.sleep.add(&r.origin, stage, &start, &end);
                }
                None => self.skipped.add("sommeil : valeur inconnue"),
            }
            return;
        }
        let short = r.kind.strip_prefix(QUANTITY_PREFIX).unwrap_or_default();
        let Ok(raw) = r.value.trim().parse::<f64>() else {
            self.skipped.add("mesure : valeur illisible");
            return;
        };
        if let Some((quantity, convert)) = activity_of(short) {
            match convert(raw, &r.unit) {
                Some(v) => self
                    .activity
                    .add(&local_day(&start), &r.origin, quantity, v),
                None => self.skipped.add(format!("unité inconnue : {}", r.unit)),
            }
        } else if let Some(m) = metric_of(short) {
            match (m.convert)(raw, &r.unit) {
                Some(value) => out.push(Out::Observation(Observation {
                    metric: m.slug.to_string(),
                    measured_at: start.utc,
                    value,
                    unit: m.unit.to_string(),
                    source: SourceId::APPLE_HEALTH,
                    external_id: Some(record_id(
                        "ah", &r.kind, start.utc, end.utc, &r.value, &r.origin,
                    )),
                    device: Some(r.origin.clone()),
                })),
                None => self.skipped.add(format!("unité inconnue : {}", r.unit)),
            }
        } else {
            self.pressure(short, raw, &start.utc, &r.origin, out);
        }
    }

    fn pressure(
        &mut self,
        short: &str,
        raw: f64,
        at: &chrono::DateTime<chrono::Utc>,
        origin: &str,
        out: &mut Vec<Out>,
    ) {
        let slot = self
            .pairs
            .entry((at.timestamp(), origin.to_string()))
            .or_default();
        match short {
            "BloodPressureSystolic" => slot.0 = Some(raw),
            _ => slot.1 = Some(raw),
        }
        if let (Some(sys), Some(dia)) = *slot {
            self.pairs.remove(&(at.timestamp(), origin.to_string()));
            let (s, d) = (sys.round() as i64, dia.round() as i64);
            out.push(Out::BloodPressure(BpRow {
                measured_at: *at,
                systolic: s,
                diastolic: d,
                external_id: record_id("ah", "bp", *at, *at, &format!("{s}/{d}"), origin),
                device: origin.to_string(),
            }));
        }
    }

    pub fn finish(mut self, out: &mut Vec<Out>) -> Skipped {
        for _ in 0..self.pairs.len() {
            self.skipped.add("tension : une seule valeur sur deux");
        }
        out.extend(self.activity.rows().into_iter().map(Out::Activity));
        let (nights, empty) = self.sleep.rows("ah");
        for _ in 0..empty {
            self.skipped.add("sommeil : nuit sans durée de sommeil");
        }
        out.extend(nights.into_iter().map(Out::Sleep));
        self.skipped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(kind: &str, unit: &str, value: &str, origin: &str, at: &str) -> Item {
        Item::Record(RawRecord {
            kind: kind.into(),
            unit: unit.into(),
            value: value.into(),
            origin: origin.into(),
            start: at.into(),
            end: at.into(),
        })
    }

    fn run(items: Vec<Item>) -> (Vec<Out>, Skipped) {
        let mut mapper = Mapper::default();
        let mut out = Vec::new();
        for i in items {
            mapper.feed(i, &mut out);
        }
        let skipped = mapper.finish(&mut out);
        (out, skipped)
    }

    fn observation(out: &[Out]) -> &Observation {
        match &out[0] {
            Out::Observation(o) => o,
            other => panic!("observation attendue : {other:?}"),
        }
    }

    #[test]
    fn wanted_types_are_the_mapped_ones_only() {
        assert!(wants("HKQuantityTypeIdentifierBodyMass"));
        assert!(wants("HKQuantityTypeIdentifierStepCount"));
        assert!(wants("HKCategoryTypeIdentifierSleepAnalysis"));
        assert!(wants("HKQuantityTypeIdentifierBloodPressureSystolic"));
        assert!(!wants("HKQuantityTypeIdentifierHeartRate"));
        assert!(!wants("HKQuantityTypeIdentifierHeight"));
        assert!(!wants("HKDataTypeSleepDurationGoal"));
    }

    #[test]
    fn imperial_weight_and_a_fraction_of_body_fat_are_converted_and_keep_their_origin() {
        let (out, skipped) = run(vec![
            rec(
                "HKQuantityTypeIdentifierBodyMass",
                "lb",
                "160",
                "Withings",
                "2026-10-25 07:30:00 +0200",
            ),
            rec(
                "HKQuantityTypeIdentifierBodyFatPercentage",
                "%",
                "0.186",
                "Withings",
                "2026-10-25 07:30:00 +0200",
            ),
        ]);
        assert_eq!(skipped.total(), 0);
        let w = observation(&out);
        assert_eq!(w.metric, "weight");
        assert!((w.value - 72.5748).abs() < 1e-3);
        assert_eq!(w.device.as_deref(), Some("Withings"));
        assert_eq!(w.source, SourceId::APPLE_HEALTH);
        assert_eq!(w.measured_at.to_rfc3339(), "2026-10-25T05:30:00+00:00");
        let Out::Observation(f) = &out[1] else {
            panic!("observation attendue")
        };
        assert!((f.value - 18.6).abs() < 1e-9);
        assert_ne!(w.external_id, f.external_id);
    }

    #[test]
    fn glucose_in_mg_dl_and_an_unknown_unit_are_handled() {
        let (out, skipped) = run(vec![
            rec(
                "HKQuantityTypeIdentifierBloodGlucose",
                "mg/dL",
                "99",
                "Libre",
                "2026-10-25 07:30:00 +0200",
            ),
            rec(
                "HKQuantityTypeIdentifierBodyMass",
                "furlong",
                "1",
                "x",
                "2026-10-25 07:30:00 +0200",
            ),
            rec(
                "HKQuantityTypeIdentifierBodyMass",
                "kg",
                "abc",
                "x",
                "2026-10-25 07:30:00 +0200",
            ),
            rec(
                "HKQuantityTypeIdentifierBodyMass",
                "kg",
                "70",
                "x",
                "pas une date",
            ),
        ]);
        assert_eq!(out.len(), 1);
        assert!((observation(&out).value - 5.4952).abs() < 1e-3);
        assert_eq!(skipped.total(), 3);
        assert_eq!(skipped.0.get("unité inconnue : furlong"), Some(&1));
    }

    #[test]
    fn systolic_and_diastolic_of_the_same_instant_make_one_reading_and_a_lone_half_is_counted() {
        let (out, skipped) = run(vec![
            rec(
                "HKQuantityTypeIdentifierBloodPressureDiastolic",
                "mmHg",
                "79",
                "Tensio",
                "2026-10-25 08:00:00 +0200",
            ),
            rec(
                "HKQuantityTypeIdentifierBloodPressureSystolic",
                "mmHg",
                "121",
                "Tensio",
                "2026-10-25 08:00:00 +0200",
            ),
            rec(
                "HKQuantityTypeIdentifierBloodPressureSystolic",
                "mmHg",
                "130",
                "Tensio",
                "2026-10-25 20:00:00 +0200",
            ),
        ]);
        assert_eq!(out.len(), 1);
        let Out::BloodPressure(bp) = &out[0] else {
            panic!("tension attendue")
        };
        assert_eq!((bp.systolic, bp.diastolic), (121, 79));
        assert_eq!(bp.device, "Tensio");
        assert_eq!(skipped.total(), 1);
    }

    #[test]
    fn steps_and_sleep_come_out_aggregated_after_the_last_record() {
        let (out, _) = run(vec![
            rec(
                "HKQuantityTypeIdentifierStepCount",
                "count",
                "1000",
                "iPhone",
                "2026-10-25 09:00:00 +0200",
            ),
            rec(
                "HKQuantityTypeIdentifierStepCount",
                "count",
                "500",
                "iPhone",
                "2026-10-25 21:00:00 +0200",
            ),
            rec(
                "HKCategoryTypeIdentifierSleepAnalysis",
                "",
                "HKCategoryValueSleepAnalysisAsleepCore",
                "Watch",
                "2026-10-25 23:30:00 +0200",
            ),
        ]);
        assert!(out
            .iter()
            .any(|o| matches!(o, Out::Activity(a) if a.steps == Some(1500))));
        assert!(out.iter().all(|o| !matches!(o, Out::Sleep(_))));
    }
}
