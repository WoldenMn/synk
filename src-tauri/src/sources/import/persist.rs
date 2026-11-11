use std::collections::BTreeMap;

use serde::Serialize;
use sqlx::SqliteConnection;

use super::rows::{
    bounded, ActivityRow, BpRow, Out, Skipped, SleepRow, WorkoutRow, MAX_DEVICE_CHARS, MAX_REASONS,
    OTHER_REASONS,
};
use super::time::utc_text;
use crate::error::AppError;
use crate::services::metrics::ObservationWriter;
use crate::sources::SourceId;
use crate::validation::check_f64;

#[derive(Debug, Default, Clone, Serialize, PartialEq, Eq)]
pub struct Tally {
    pub observations: u64,
    pub observations_ignored: u64,
    pub blood_pressure: u64,
    pub sleep: u64,
    pub activity: u64,
    pub workouts: u64,
    pub devices: BTreeMap<String, u64>,
}

impl Tally {
    fn device(&mut self, name: &str) {
        if name.is_empty() {
            return;
        }
        let name = bounded(name, MAX_DEVICE_CHARS);
        let key = if self.devices.contains_key(&name) || self.devices.len() < MAX_REASONS {
            name
        } else {
            OTHER_REASONS.to_string()
        };
        *self.devices.entry(key).or_insert(0) += 1;
    }

    pub fn written(&self) -> u64 {
        self.observations + self.blood_pressure + self.sleep + self.activity + self.workouts
    }
}

pub struct Persister {
    source: SourceId,
    observations: ObservationWriter,
    pub tally: Tally,
    pub skipped: Skipped,
}

impl Persister {
    pub fn new(source: SourceId) -> Self {
        Self {
            source,
            observations: ObservationWriter::default(),
            tally: Tally::default(),
            skipped: Skipped::default(),
        }
    }

    pub async fn write(&mut self, conn: &mut SqliteConnection, out: Out) -> Result<(), AppError> {
        match out.with_bounded_device() {
            Out::Observation(o) => {
                let device = o.device.clone().unwrap_or_default();
                match self.observations.write(conn, &o).await {
                    Ok(true) => {
                        self.tally.observations += 1;
                        self.tally.device(&device);
                    }
                    Ok(false) => self.tally.observations_ignored += 1,
                    Err(AppError::Validation(_)) => {
                        self.skipped
                            .add(format!("{} : valeur hors bornes", o.metric));
                    }
                    Err(other) => return Err(other),
                }
            }
            Out::BloodPressure(r) => self.blood_pressure(conn, &r).await?,
            Out::Sleep(r) => {
                if upsert_sleep(conn, self.source, &r).await? {
                    self.tally.sleep += 1;
                    self.tally.device(&r.device);
                }
            }
            Out::Activity(r) => {
                if upsert_activity(conn, self.source, &r).await? {
                    self.tally.activity += 1;
                    self.tally.device(&r.device);
                }
            }
            Out::Workout(r) => {
                if insert_workout(conn, self.source, &r).await? {
                    self.tally.workouts += 1;
                    self.tally.device(&r.device);
                }
            }
        }
        Ok(())
    }

    async fn blood_pressure(
        &mut self,
        conn: &mut SqliteConnection,
        r: &BpRow,
    ) -> Result<(), AppError> {
        let valid = check_f64("systolic", r.systolic as f64, 50.0, 300.0).is_ok()
            && check_f64("diastolic", r.diastolic as f64, 30.0, 200.0).is_ok();
        if !valid {
            self.skipped.add("tension : valeur hors bornes");
            return Ok(());
        }
        let done = sqlx::query(
            "INSERT INTO blood_pressure (measured_at, source, systolic, diastolic, external_id, device) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6) ON CONFLICT DO NOTHING",
        )
        .bind(utc_text(r.measured_at))
        .bind(self.source.as_str())
        .bind(r.systolic)
        .bind(r.diastolic)
        .bind(&r.external_id)
        .bind(&r.device)
        .execute(&mut *conn)
        .await?;
        if done.rows_affected() == 1 {
            self.tally.blood_pressure += 1;
            self.tally.device(&r.device);
        }
        Ok(())
    }
}

async fn upsert_sleep(
    conn: &mut SqliteConnection,
    source: SourceId,
    r: &SleepRow,
) -> Result<bool, AppError> {
    let done = sqlx::query(
        "INSERT INTO sleep_records (date, source, external_id, start_at, end_at, duration_total, \
         duration_deep, duration_light, duration_rem, duration_awake, total_timeinbed, device) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12) \
         ON CONFLICT(date, source) DO UPDATE SET external_id = excluded.external_id, \
         start_at = excluded.start_at, end_at = excluded.end_at, \
         duration_total = excluded.duration_total, duration_deep = excluded.duration_deep, \
         duration_light = excluded.duration_light, duration_rem = excluded.duration_rem, \
         duration_awake = excluded.duration_awake, total_timeinbed = excluded.total_timeinbed, \
         device = excluded.device \
         WHERE sleep_records.start_at IS NOT excluded.start_at \
         OR sleep_records.end_at IS NOT excluded.end_at \
         OR sleep_records.duration_total IS NOT excluded.duration_total \
         OR sleep_records.duration_deep IS NOT excluded.duration_deep \
         OR sleep_records.duration_light IS NOT excluded.duration_light \
         OR sleep_records.duration_rem IS NOT excluded.duration_rem \
         OR sleep_records.duration_awake IS NOT excluded.duration_awake \
         OR sleep_records.total_timeinbed IS NOT excluded.total_timeinbed \
         OR sleep_records.device IS NOT excluded.device",
    )
    .bind(&r.date)
    .bind(source.as_str())
    .bind(&r.external_id)
    .bind(&r.start_at)
    .bind(&r.end_at)
    .bind(r.duration_total)
    .bind(r.duration_deep)
    .bind(r.duration_light)
    .bind(r.duration_rem)
    .bind(r.duration_awake)
    .bind(r.total_timeinbed)
    .bind(&r.device)
    .execute(&mut *conn)
    .await?;
    Ok(done.rows_affected() == 1)
}

async fn upsert_activity(
    conn: &mut SqliteConnection,
    source: SourceId,
    r: &ActivityRow,
) -> Result<bool, AppError> {
    let done = sqlx::query(
        "INSERT INTO activity (date, source, external_id, steps, distance, calories_active, \
         calories_total, device) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8) \
         ON CONFLICT(date, source) DO UPDATE SET external_id = excluded.external_id, \
         steps = excluded.steps, distance = excluded.distance, \
         calories_active = excluded.calories_active, calories_total = excluded.calories_total, \
         device = excluded.device \
         WHERE activity.steps IS NOT excluded.steps \
         OR activity.distance IS NOT excluded.distance \
         OR activity.calories_active IS NOT excluded.calories_active \
         OR activity.calories_total IS NOT excluded.calories_total \
         OR activity.device IS NOT excluded.device",
    )
    .bind(&r.date)
    .bind(source.as_str())
    .bind(&r.external_id)
    .bind(r.steps)
    .bind(r.distance)
    .bind(r.calories_active)
    .bind(r.calories_total)
    .bind(&r.device)
    .execute(&mut *conn)
    .await?;
    Ok(done.rows_affected() == 1)
}

async fn insert_workout(
    conn: &mut SqliteConnection,
    source: SourceId,
    r: &WorkoutRow,
) -> Result<bool, AppError> {
    let done = sqlx::query(
        "INSERT INTO workouts (date, source, external_id, start_at, end_at, category_label, \
         duration, distance, calories, device, elevation, hr_average, hr_min, hr_max) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14) \
         ON CONFLICT DO NOTHING",
    )
    .bind(&r.date)
    .bind(source.as_str())
    .bind(&r.external_id)
    .bind(&r.start_at)
    .bind(&r.end_at)
    .bind(&r.category_label)
    .bind(r.duration)
    .bind(r.distance)
    .bind(r.calories)
    .bind(&r.device)
    .bind(r.elevation)
    .bind(r.hr_average)
    .bind(r.hr_min)
    .bind(r.hr_max)
    .execute(&mut *conn)
    .await?;
    Ok(done.rows_affected() == 1)
}
