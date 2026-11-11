use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};
use fitparser::profile::MesgNum;
use fitparser::{FitDataRecord, Value};

use crate::sources::import::fingerprint::record_id;
use crate::sources::import::rows::{BpRow, Out, Skipped, WorkoutRow};
use crate::sources::import::sleep::{SleepAgg, Stage};
use crate::sources::import::time::{local_day, utc_text, Stamp};
use crate::sources::import::trace::row::sport_label;
use crate::sources::{Observation, SourceId};

const PREFIX: &str = "fit";

struct Fields<'a>(HashMap<&'a str, &'a Value>);

impl<'a> Fields<'a> {
    fn of(record: &'a FitDataRecord) -> Self {
        Self(
            record
                .fields()
                .iter()
                .map(|f| (f.name(), f.value()))
                .collect(),
        )
    }

    fn number(&self, name: &str) -> Option<f64> {
        match self.0.get(name)? {
            Value::Array(_) | Value::String(_) | Value::Invalid | Value::Timestamp(_) => None,
            other => {
                let value: Option<f64> = (*other).clone().try_into().ok();
                value.filter(|v| v.is_finite())
            }
        }
    }

    fn time(&self, name: &str) -> Option<DateTime<Utc>> {
        match self.0.get(name)? {
            Value::Timestamp(t) => Some(t.with_timezone(&Utc)),
            _ => None,
        }
    }

    fn text(&self, name: &str) -> Option<&str> {
        match self.0.get(name)? {
            Value::String(s) if !s.is_empty() => Some(s.as_str()),
            _ => None,
        }
    }
}

fn weight_kg(fields: &Fields<'_>) -> Option<f64> {
    match fields.0.get("weight")? {
        Value::SInt64(raw) => Some(*raw as f64 / 100.0),
        _ => fields.number("weight"),
    }
}

fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    chars
        .next()
        .map(|c| c.to_uppercase().collect::<String>() + chars.as_str())
        .unwrap_or_default()
}

#[derive(Default)]
pub struct Mapper {
    device: String,
    sleep: SleepAgg,
    levels: Vec<(DateTime<Utc>, Stage)>,
    last_time: Option<DateTime<Utc>>,
    out: Vec<Out>,
    pub skipped: Skipped,
}

impl Mapper {
    pub fn feed(&mut self, record: &FitDataRecord) {
        let fields = Fields::of(record);
        if let Some(t) = fields.time("timestamp") {
            self.last_time = Some(self.last_time.map_or(t, |l| l.max(t)));
        }
        match record.kind() {
            MesgNum::FileId => self.file_id(&fields),
            MesgNum::Session => self.session(&fields),
            MesgNum::WeightScale => self.weight_scale(&fields),
            MesgNum::BloodPressure => self.pressure(&fields),
            MesgNum::SleepLevel => self.level(&fields),
            _ => {}
        }
    }

    fn file_id(&mut self, f: &Fields<'_>) {
        if !self.device.is_empty() {
            return;
        }
        let maker = f.text("manufacturer").map(capitalize).unwrap_or_default();
        let product = f.text("product_name").unwrap_or_default();
        self.device = format!("{maker} {product}").trim().to_string();
    }

    fn session(&mut self, f: &Fields<'_>) {
        let elapsed = f.number("total_elapsed_time").unwrap_or(0.0);
        let end = f.time("timestamp");
        let start = f
            .time("start_time")
            .or_else(|| end.map(|e| e - Duration::seconds(elapsed.round() as i64)));
        let (Some(start), Some(stop)) = (
            start,
            end.or_else(|| start.map(|s| s + Duration::seconds(elapsed.round() as i64))),
        ) else {
            self.skipped.add("séance : date illisible");
            return;
        };
        let sport = f.text("sport").unwrap_or_default();
        let sub = f.text("sub_sport").unwrap_or_default();
        let label_source = if sport == "training" && !sub.is_empty() && sub != "generic" {
            sub
        } else {
            sport
        };
        let timer = f.number("total_timer_time").filter(|v| *v > 0.0);
        let seconds = timer.unwrap_or(elapsed).round() as i64;
        let signature = format!(
            "{seconds}|{}",
            f.number("total_distance").unwrap_or(0.0).round()
        );
        let int = |name: &str| f.number(name).map(|v| v.round() as i64);
        self.out.push(Out::Workout(WorkoutRow {
            date: local_day(&Stamp::on_this_computer(start)),
            external_id: record_id(PREFIX, "session", start, stop, &signature, sport),
            start_at: utc_text(start),
            end_at: utc_text(stop),
            category_label: sport_label(label_source),
            duration: seconds,
            distance: f.number("total_distance").filter(|v| *v > 0.0),
            calories: f.number("total_calories").filter(|v| *v > 0.0),
            elevation: f.number("total_ascent").filter(|v| *v > 0.0),
            hr_average: int("avg_heart_rate"),
            hr_min: int("min_heart_rate"),
            hr_max: int("max_heart_rate"),
            device: self.device.clone(),
        }));
    }

    fn observation(&mut self, metric: &str, unit: &str, at: DateTime<Utc>, value: f64) {
        let text = format!("{value:.4}");
        self.out.push(Out::Observation(Observation {
            metric: metric.to_string(),
            measured_at: at,
            value,
            unit: unit.to_string(),
            source: SourceId::FIT,
            external_id: Some(record_id(PREFIX, metric, at, at, &text, &self.device)),
            device: Some(self.device.clone()).filter(|d| !d.is_empty()),
        }));
    }

    fn weight_scale(&mut self, f: &Fields<'_>) {
        let Some(at) = f.time("timestamp") else {
            self.skipped.add("pesée : date illisible");
            return;
        };
        let weight = weight_kg(f);
        if let Some(w) = weight {
            self.observation("weight", "kg", at, w);
        }
        if let Some(v) = f.number("percent_fat") {
            self.observation("fat_ratio", "%", at, v);
        }
        if let (Some(w), Some(pct)) = (weight, f.number("percent_hydration")) {
            self.observation("hydration", "kg", at, w * pct / 100.0);
        }
        if let Some(v) = f.number("muscle_mass") {
            self.observation("muscle_mass", "kg", at, v);
        }
        if let Some(v) = f.number("bone_mass") {
            self.observation("bone_mass", "kg", at, v);
        }
        if let Some(v) = f.number("bmi") {
            self.observation("bmi", "kg/m2", at, v);
        }
        if weight.is_none() {
            self.skipped.add("pesée : poids absent");
        }
    }

    fn pressure(&mut self, f: &Fields<'_>) {
        let (Some(at), Some(sys), Some(dia)) = (
            f.time("timestamp"),
            f.number("systolic_pressure"),
            f.number("diastolic_pressure"),
        ) else {
            self.skipped.add("tension : valeur ou date manquante");
            return;
        };
        let text = format!("{sys}/{dia}");
        self.out.push(Out::BloodPressure(BpRow {
            measured_at: at,
            systolic: sys.round() as i64,
            diastolic: dia.round() as i64,
            external_id: record_id(PREFIX, "bp", at, at, &text, &self.device),
            device: self.device.clone(),
        }));
    }

    fn level(&mut self, f: &Fields<'_>) {
        let (Some(at), Some(name)) = (f.time("timestamp"), f.text("sleep_level")) else {
            self.skipped.add("sommeil : niveau ou date manquant");
            return;
        };
        let stage = match name {
            "awake" => Stage::Awake,
            "light" => Stage::Core,
            "deep" => Stage::Deep,
            "rem" => Stage::Rem,
            _ => Stage::InBed,
        };
        self.levels.push((at, stage));
    }

    pub fn finish(mut self) -> (Vec<Out>, Skipped) {
        self.levels.sort_by_key(|(t, _)| *t);
        let end_of_file = self.last_time;
        let device = self.device.clone();
        for (i, (start, stage)) in self.levels.iter().enumerate() {
            if *stage == Stage::InBed {
                continue;
            }
            let next = self.levels.get(i + 1).map(|(t, _)| *t).or(end_of_file);
            match next {
                Some(end) if end > *start => {
                    let (a, b) = (
                        Stamp::on_this_computer(*start),
                        Stamp::on_this_computer(end),
                    );
                    self.sleep.add(&device, *stage, &a, &b);
                }
                _ => self.skipped.add("sommeil : dernier niveau sans fin"),
            }
        }
        let (nights, empty) = self.sleep.rows(PREFIX);
        if empty > 0 {
            self.skipped
                .0
                .insert("sommeil : nuit sans sommeil".into(), empty);
        }
        self.out.extend(nights.into_iter().map(Out::Sleep));
        (self.out, self.skipped)
    }
}
