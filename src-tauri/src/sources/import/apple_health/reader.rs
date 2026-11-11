use std::io::BufRead;
use std::sync::atomic::{AtomicBool, Ordering};

use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, XmlVersion};

use super::items::{Item, RawRecord, RawStat, RawWorkout};
use crate::error::AppError;
use crate::sources::import::limits::{too_big, Guarded, Limits};

const CANCEL_CHECK_EVERY: u32 = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseEnd {
    Done,
    Cancelled,
}

fn invalid(e: impl std::fmt::Display) -> AppError {
    AppError::Validation(format!("export Apple Santé illisible : {e}"))
}

fn attribute(e: &BytesStart<'_>, name: &str) -> Result<Option<String>, AppError> {
    for attr in e.attributes() {
        let attr = attr.map_err(invalid)?;
        if attr.key.as_ref() == name {
            return Ok(Some(
                attr.normalized_value(XmlVersion::Implicit1_0)
                    .map_err(invalid)?
                    .into_owned(),
            ));
        }
    }
    Ok(None)
}

fn record_of(e: &BytesStart<'_>) -> Result<RawRecord, AppError> {
    let mut r = RawRecord::default();
    for attr in e.attributes() {
        let attr = attr.map_err(invalid)?;
        let slot = match attr.key.as_ref() {
            "type" => &mut r.kind,
            "unit" => &mut r.unit,
            "value" => &mut r.value,
            "sourceName" => &mut r.origin,
            "startDate" => &mut r.start,
            "endDate" => &mut r.end,
            _ => continue,
        };
        *slot = attr
            .normalized_value(XmlVersion::Implicit1_0)
            .map_err(invalid)?
            .into_owned();
    }
    Ok(r)
}

fn workout_of(e: &BytesStart<'_>) -> Result<RawWorkout, AppError> {
    let mut w = RawWorkout::default();
    for attr in e.attributes() {
        let attr = attr.map_err(invalid)?;
        let slot = match attr.key.as_ref() {
            "workoutActivityType" => &mut w.activity,
            "sourceName" => &mut w.origin,
            "startDate" => &mut w.start,
            "endDate" => &mut w.end,
            "duration" => &mut w.duration,
            "durationUnit" => &mut w.duration_unit,
            "totalDistance" => &mut w.distance,
            "totalDistanceUnit" => &mut w.distance_unit,
            "totalEnergyBurned" => &mut w.energy,
            "totalEnergyBurnedUnit" => &mut w.energy_unit,
            _ => continue,
        };
        *slot = attr
            .normalized_value(XmlVersion::Implicit1_0)
            .map_err(invalid)?
            .into_owned();
    }
    Ok(w)
}

fn stat_of(e: &BytesStart<'_>) -> Result<RawStat, AppError> {
    Ok(RawStat {
        kind: attribute(e, "type")?.unwrap_or_default(),
        sum: attribute(e, "sum")?.unwrap_or_default(),
        unit: attribute(e, "unit")?.unwrap_or_default(),
    })
}

pub fn parse_export<R, W, F>(
    source: R,
    wants: W,
    cancel: &AtomicBool,
    on_item: F,
) -> Result<ParseEnd, AppError>
where
    R: BufRead,
    W: Fn(&str) -> bool,
    F: FnMut(Item) -> Result<(), AppError>,
{
    parse_export_limited(source, &Limits::APPLE_HEALTH, wants, cancel, on_item)
}

pub fn parse_export_limited<R, W, F>(
    source: R,
    limits: &Limits,
    wants: W,
    cancel: &AtomicBool,
    mut on_item: F,
) -> Result<ParseEnd, AppError>
where
    R: BufRead,
    W: Fn(&str) -> bool,
    F: FnMut(Item) -> Result<(), AppError>,
{
    let (guarded, gauge) = Guarded::new(source, limits.node);
    let mut reader = Reader::from_reader(guarded);
    let mut elements: u64 = 0;
    reader.config_mut().check_end_names = false;
    let mut buf = Vec::with_capacity(4096);
    let mut depth: u32 = 0;
    let mut workout: Option<RawWorkout> = None;
    let mut ticks: u32 = 0;
    loop {
        ticks = ticks.wrapping_add(1);
        if ticks.is_multiple_of(CANCEL_CHECK_EVERY) && cancel.load(Ordering::Relaxed) {
            return Ok(ParseEnd::Cancelled);
        }
        let event = reader.read_event_into(&mut buf).map_err(invalid)?;
        gauge.event_done();
        if matches!(event, Event::Start(_) | Event::Empty(_)) {
            elements += 1;
            if elements > limits.elements {
                return Err(too_big("export Apple Santé"));
            }
        }
        match event {
            Event::Eof => break,
            Event::Start(e) => {
                if depth >= limits.depth {
                    return Err(invalid("éléments imbriqués trop profondément"));
                }
                if depth == 1 {
                    match e.name().as_ref() {
                        "Record" => {
                            let r = record_of(&e)?;
                            if wants(&r.kind) {
                                on_item(Item::Record(r))?;
                            }
                        }
                        "Workout" => workout = Some(workout_of(&e)?),
                        _ => {}
                    }
                }
                depth += 1;
            }
            Event::Empty(e) => match (depth, e.name().as_ref()) {
                (1, "Record") => {
                    let r = record_of(&e)?;
                    if wants(&r.kind) {
                        on_item(Item::Record(r))?;
                    }
                }
                (1, "Workout") => on_item(Item::Workout(workout_of(&e)?))?,
                (2, "WorkoutStatistics") => {
                    if let Some(w) = workout.as_mut() {
                        w.stats.push(stat_of(&e)?);
                    }
                }
                _ => {}
            },
            Event::End(e) => {
                depth = depth.saturating_sub(1);
                if depth == 1 && e.name().as_ref() == "Workout" {
                    if let Some(w) = workout.take() {
                        on_item(Item::Workout(w))?;
                    }
                }
            }
            _ => {}
        }
        buf.clear();
    }
    Ok(ParseEnd::Done)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    const SAMPLE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE HealthData [
<!-- HealthKit Export Version: 14 -->
<!ELEMENT HealthData (ExportDate,Me,(Record|Correlation|Workout|ActivitySummary)*)>
<!ATTLIST HealthData
  locale CDATA #REQUIRED
>
<!ELEMENT Record (MetadataEntry*)>
<!ELEMENT MetadataEntry EMPTY>
]>
<HealthData locale="fr_FR">
 <ExportDate value="2026-10-25 23:30:00 +0200"/>
 <Me HKCharacteristicTypeIdentifierDateOfBirth="1990-01-01"/>
 <Record type="HKQuantityTypeIdentifierBodyMass" sourceName="Balance de Camille" sourceVersion="1" unit="kg" creationDate="2026-10-25 07:31:02 +0200" startDate="2026-10-25 07:30:00 +0200" endDate="2026-10-25 07:30:00 +0200" value="72.4">
  <MetadataEntry key="HKWasUserEntered" value="1"/>
 </Record>
 <Record type="HKQuantityTypeIdentifierHeartRate" sourceName="Apple Watch" unit="count/min" startDate="2026-10-25 09:14:00 +0200" endDate="2026-10-25 09:14:00 +0200" value="62"/>
 <Correlation type="HKCorrelationTypeIdentifierBloodPressure" sourceName="Tensio" startDate="2026-10-25 08:00:00 +0200" endDate="2026-10-25 08:00:00 +0200">
  <Record type="HKQuantityTypeIdentifierBloodPressureSystolic" sourceName="Tensio" unit="mmHg" startDate="2026-10-25 08:00:00 +0200" endDate="2026-10-25 08:00:00 +0200" value="121"/>
 </Correlation>
 <Record type="HKQuantityTypeIdentifierBloodPressureSystolic" sourceName="Tensio" unit="mmHg" startDate="2026-10-25 08:00:00 +0200" endDate="2026-10-25 08:00:00 +0200" value="121"/>
 <Workout workoutActivityType="HKWorkoutActivityTypeRunning" duration="30.5" durationUnit="min" totalDistance="5.2" totalDistanceUnit="km" sourceName="Apple Watch de &quot;Camille&quot;" startDate="2026-10-25 18:00:00 +0200" endDate="2026-10-25 18:30:30 +0200">
  <WorkoutEvent type="HKWorkoutEventTypePause" date="2026-10-25 18:10:00 +0200"/>
  <WorkoutStatistics type="HKQuantityTypeIdentifierActiveEnergyBurned" sum="320" unit="kcal"/>
 </Workout>
 <Workout workoutActivityType="HKWorkoutActivityTypeYoga" sourceName="Apple Watch" startDate="2026-10-24 18:00:00 +0200" endDate="2026-10-24 18:20:00 +0200"/>
 <ActivitySummary dateComponents="2026-10-25" activeEnergyBurned="400"/>
</HealthData>"#;

    fn collect(xml: &str, wants: impl Fn(&str) -> bool) -> Vec<Item> {
        let mut out = Vec::new();
        let cancel = AtomicBool::new(false);
        let end = parse_export(Cursor::new(xml.as_bytes()), wants, &cancel, |i| {
            out.push(i);
            Ok(())
        })
        .expect("lecture");
        assert_eq!(end, ParseEnd::Done);
        out
    }

    #[test]
    fn records_and_workouts_are_read_through_the_dtd_and_unwanted_types_are_skipped() {
        let items = collect(SAMPLE, |k| k != "HKQuantityTypeIdentifierHeartRate");
        let kinds: Vec<String> = items
            .iter()
            .map(|i| match i {
                Item::Record(r) => r.kind.clone(),
                Item::Workout(w) => w.activity.clone(),
            })
            .collect();
        assert_eq!(
            kinds,
            vec![
                "HKQuantityTypeIdentifierBodyMass",
                "HKQuantityTypeIdentifierBloodPressureSystolic",
                "HKWorkoutActivityTypeRunning",
                "HKWorkoutActivityTypeYoga",
            ]
        );
    }

    #[test]
    fn a_record_carries_its_value_unit_origin_and_dates() {
        let items = collect(SAMPLE, |k| k.ends_with("BodyMass"));
        let Item::Record(r) = &items[0] else {
            panic!("record attendu")
        };
        assert_eq!(r.value, "72.4");
        assert_eq!(r.unit, "kg");
        assert_eq!(r.origin, "Balance de Camille");
        assert_eq!(r.start, "2026-10-25 07:30:00 +0200");
        assert_eq!(r.end, "2026-10-25 07:30:00 +0200");
    }

    #[test]
    fn records_nested_in_a_correlation_are_not_read_twice() {
        let items = collect(SAMPLE, |k| k.ends_with("BloodPressureSystolic"));
        let records = items
            .iter()
            .filter(|i| matches!(i, Item::Record(_)))
            .count();
        assert_eq!(records, 1);
    }

    #[test]
    fn a_workout_keeps_its_statistics_and_unescaped_origin() {
        let items = collect(SAMPLE, |_| false);
        let Item::Workout(w) = &items[0] else {
            panic!("séance attendue")
        };
        assert_eq!(w.origin, "Apple Watch de \"Camille\"");
        assert_eq!(w.duration, "30.5");
        assert_eq!(w.distance_unit, "km");
        assert_eq!(w.stats.len(), 1);
        assert_eq!(w.stats[0].sum, "320");
        let Item::Workout(y) = &items[1] else {
            panic!("séance attendue")
        };
        assert!(y.stats.is_empty());
    }

    #[test]
    fn a_cancel_flag_stops_the_read_and_a_broken_document_is_an_error() {
        let big = format!(
            "<HealthData>{}</HealthData>",
            "<Record type=\"T\" unit=\"u\" value=\"1\" sourceName=\"s\" startDate=\"a\" endDate=\"b\"/>"
                .repeat(10_000)
        );
        let cancel = AtomicBool::new(true);
        let mut seen = 0;
        let end = parse_export(
            Cursor::new(big.as_bytes()),
            |_| true,
            &cancel,
            |_| {
                seen += 1;
                Ok(())
            },
        )
        .expect("annulation");
        assert_eq!(end, ParseEnd::Cancelled);
        assert!(seen < 10_000, "{seen}");

        let cancel = AtomicBool::new(false);
        let broken = parse_export(
            Cursor::new(b"<HealthData><Record type=\"T\" =oops/></HealthData>".as_slice()),
            |_| true,
            &cancel,
            |_| Ok(()),
        );
        assert!(broken.is_err());
    }

    #[test]
    fn a_giant_attribute_deep_nesting_and_too_many_elements_are_refused() {
        let tight = Limits {
            bytes: 1024,
            node: 512,
            depth: 6,
            elements: 40,
        };
        let run = |xml: String| {
            parse_export_limited(
                Cursor::new(xml.into_bytes()),
                &tight,
                |_| true,
                &AtomicBool::new(false),
                |_| Ok(()),
            )
        };
        let giant = format!(
            "<HealthData><Record type=\"t\" sourceName=\"{}\"/></HealthData>",
            "x".repeat(20_000)
        );
        let err = run(giant).unwrap_err();
        assert!(err.to_string().contains("volumineux"), "{err}");

        let deep = format!(
            "<HealthData>{}{}</HealthData>",
            "<a>".repeat(20),
            "</a>".repeat(20)
        );
        let err = run(deep).unwrap_err();
        assert!(err.to_string().contains("profondément"), "{err}");

        let many = format!("<HealthData>{}</HealthData>", "<Me/>".repeat(100));
        let err = run(many).unwrap_err();
        assert!(err.to_string().contains("limite"), "{err}");
    }
}
