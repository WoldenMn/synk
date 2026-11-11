use super::config::{ColumnMap, CsvConfig};
use super::dates;
use super::read::{parse_number, rows};
use super::targets::{self, Kind, Target};
use crate::error::AppError;
use crate::sources::import::fingerprint::record_id;
use crate::sources::import::rows::{BpRow, Out, Skipped};
use crate::sources::{Observation, SourceId};

pub struct Converted {
    pub outs: Vec<Out>,
    pub skipped: Skipped,
}

struct Bound<'a> {
    index: usize,
    map: &'a ColumnMap,
    target: &'static Target,
}

fn bind<'a>(header: &[String], config: &'a CsvConfig) -> Result<Vec<Bound<'a>>, AppError> {
    let position = |name: &str| header.iter().position(|h| h == name);
    config
        .columns
        .iter()
        .map(|map| {
            let index = position(&map.column).ok_or_else(|| {
                AppError::Validation(format!("colonne introuvable : {}", map.column))
            })?;
            let target = targets::find(&map.metric)
                .ok_or_else(|| AppError::Validation(format!("mesure inconnue : {}", map.metric)))?;
            Ok(Bound { index, map, target })
        })
        .collect()
}

pub fn convert(text: &str, config: &CsvConfig) -> Result<Converted, AppError> {
    let all = rows(text, config.delimiter)?;
    let header = all
        .get(config.skip_rows)
        .ok_or_else(|| AppError::Validation("fichier CSV vide ou sans ligne d'en-tête".into()))?;
    config.validate(header)?;
    let bound = bind(header, config)?;
    let position = |name: &str| header.iter().position(|h| h == name);
    let date_index = position(&config.date_column).unwrap_or(0);
    let time_index = config.time_column.as_deref().and_then(position);
    let mut outs = Vec::new();
    let mut skipped = Skipped::default();
    for row in all.iter().skip(config.skip_rows + 1) {
        let mut when = row.get(date_index).cloned().unwrap_or_default();
        if let Some(i) = time_index {
            when = format!("{when} {}", row.get(i).map_or("", String::as_str));
        }
        let Some(at) = dates::parse(&when, &config.date_format) else {
            skipped.add("ligne : date illisible");
            continue;
        };
        let (mut systolic, mut diastolic) = (None, None);
        for b in &bound {
            let raw = row.get(b.index).map_or("", String::as_str);
            if raw.is_empty() {
                continue;
            }
            let Some(number) = parse_number(raw, config.decimal) else {
                skipped.add(format!("{} : valeur illisible", b.target.label));
                continue;
            };
            let Some(value) = targets::convert(&b.map.metric, &b.map.unit, number) else {
                skipped.add(format!("{} : unité impossible", b.target.label));
                continue;
            };
            match b.target.kind {
                Kind::Systolic => systolic = Some(value),
                Kind::Diastolic => diastolic = Some(value),
                Kind::Observation => outs.push(Out::Observation(Observation {
                    metric: b.map.metric.clone(),
                    measured_at: at,
                    value,
                    unit: b.target.stored_unit.to_string(),
                    source: SourceId::CSV,
                    external_id: Some(record_id("csv", &b.map.metric, at, at, raw, &b.map.column)),
                    device: None,
                })),
            }
        }
        match (systolic, diastolic) {
            (Some(s), Some(d)) => outs.push(Out::BloodPressure(BpRow {
                measured_at: at,
                systolic: s.round() as i64,
                diastolic: d.round() as i64,
                external_id: record_id("csv", "bp", at, at, &format!("{s}/{d}"), ""),
                device: String::new(),
            })),
            (None, None) => {}
            _ => skipped.add("tension : une seule des deux valeurs"),
        }
    }
    Ok(Converted { outs, skipped })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> CsvConfig {
        CsvConfig {
            delimiter: ';',
            decimal: ',',
            skip_rows: 0,
            date_column: "Date".into(),
            time_column: Some("Heure".into()),
            date_format: "%d/%m/%Y %H:%M".into(),
            columns: vec![
                ColumnMap {
                    column: "Poids (lb)".into(),
                    metric: "weight".into(),
                    unit: "lb".into(),
                },
                ColumnMap {
                    column: "Sys".into(),
                    metric: "systolic".into(),
                    unit: "mmHg".into(),
                },
                ColumnMap {
                    column: "Dia".into(),
                    metric: "diastolic".into(),
                    unit: "mmHg".into(),
                },
            ],
        }
    }

    const FILE: &str = "Date;Heure;Poids (lb);Sys;Dia\n25/10/2026;07:30;160,0;121;79\n26/10/2026;07:45;158,0;118;\n27/10/2026;07:50;abc;;\nhier;07:00;150;120;80\n";

    #[test]
    fn rows_become_observations_pressures_and_counted_skips() {
        let done = convert(FILE, &config()).expect("conversion");
        let weights: Vec<_> = done
            .outs
            .iter()
            .filter_map(|o| match o {
                Out::Observation(o) => Some(o),
                _ => None,
            })
            .collect();
        assert_eq!(weights.len(), 2);
        assert!((weights[0].value - 72.5748).abs() < 1e-3);
        assert_eq!(weights[0].unit, "kg");
        let pressures = done
            .outs
            .iter()
            .filter(|o| matches!(o, Out::BloodPressure(_)))
            .count();
        assert_eq!(pressures, 1);
        assert_eq!(done.skipped.0.get("ligne : date illisible"), Some(&1));
        assert_eq!(done.skipped.0.get("Poids : valeur illisible"), Some(&1));
        assert_eq!(
            done.skipped.0.get("tension : une seule des deux valeurs"),
            Some(&1)
        );
    }

    #[test]
    fn the_same_row_always_has_the_same_id() {
        let a = convert(FILE, &config()).unwrap();
        let b = convert(FILE, &config()).unwrap();
        let ids = |c: &Converted| {
            c.outs
                .iter()
                .filter_map(|o| match o {
                    Out::Observation(o) => o.external_id.clone(),
                    _ => None,
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(ids(&a), ids(&b));
        assert_ne!(ids(&a)[0], ids(&a)[1]);
    }

    #[test]
    fn a_header_that_does_not_match_the_configuration_is_an_error() {
        let err = convert("Jour;Valeur\n1;2\n", &config()).err().unwrap();
        assert!(err.to_string().contains("introuvable"), "{err}");
        let err = convert("", &config()).err().unwrap();
        assert!(err.to_string().contains("vide"), "{err}");
    }
}
