use serde::Serialize;

use super::config::{signature, ColumnMap, CsvConfig};
use super::dates::{self, DateFormat};
use super::read::{decode_text, detect_delimiter, header_row, rows};
use super::targets::{self, Kind};
use crate::error::AppError;

const SAMPLE_ROWS: usize = 5;
const GUESS_ROWS: usize = 40;

#[derive(Debug, Clone, Serialize)]
pub struct Inspection {
    pub header: Vec<String>,
    pub sample: Vec<Vec<String>>,
    pub row_count: usize,
    pub signature: String,
    pub suggestion: CsvConfig,
    pub preset: Option<&'static str>,
    pub formats: Vec<DateFormat>,
}

fn contains_any(name: &str, keywords: &[&str]) -> bool {
    let lower = name.to_lowercase();
    keywords.iter().any(|k| lower.contains(k))
}

fn guess_unit(column: &str, target: &targets::Target) -> &'static str {
    let lower = column.to_lowercase();
    let hinted = target.units.iter().find(|u| {
        let id = u.id.to_lowercase();
        let mentioned = lower.contains(&format!("({id}"))
            || lower.contains(&format!("[{id}"))
            || lower.ends_with(&format!(" {id}"));
        mentioned && id != "%" && id != "count"
    });
    hinted.map_or(target.units[0].id, |u| u.id)
}

fn guess_columns(header: &[String], skip: &[&str]) -> Vec<ColumnMap> {
    let mut maps: Vec<ColumnMap> = Vec::new();
    for name in header {
        if skip.contains(&name.as_str()) || name.is_empty() {
            continue;
        }
        let found = targets::TARGETS
            .iter()
            .filter(|t| contains_any(name, t.keywords))
            .find(|t| {
                !maps
                    .iter()
                    .any(|m| m.metric == t.metric && t.kind != Kind::Observation)
            });
        if let Some(t) = found {
            maps.push(ColumnMap {
                column: name.clone(),
                metric: t.metric.to_string(),
                unit: guess_unit(name, t).to_string(),
            });
        }
    }
    maps
}

fn libreview(header: &[String]) -> Option<Vec<ColumnMap>> {
    let has = |n: &str| header.iter().any(|h| h == n);
    if !(has("Device Timestamp") && has("Record Type") && has("Historic Glucose mg/dL")) {
        return None;
    }
    let maps: Vec<ColumnMap> = [
        "Historic Glucose mg/dL",
        "Scan Glucose mg/dL",
        "Strip Glucose mg/dL",
    ]
    .iter()
    .filter(|c| has(c))
    .map(|c| ColumnMap {
        column: (*c).to_string(),
        metric: "blood_glucose".into(),
        unit: "mg/dL".into(),
    })
    .collect();
    Some(maps)
}

fn date_column(header: &[String], data: &[Vec<String>]) -> (String, Option<String>) {
    let by_name = header
        .iter()
        .position(|h| contains_any(h, &["date", "time", "heure", "timestamp", "datum"]));
    let by_values = || {
        (0..header.len()).find(|i| {
            let cells = data.iter().filter_map(|r| r.get(*i)).map(String::as_str);
            dates::guess(cells).is_some()
        })
    };
    let index = by_name.or_else(by_values).unwrap_or(0);
    let name = header.get(index).cloned().unwrap_or_default();
    let time = header
        .iter()
        .enumerate()
        .find(|(i, h)| {
            *i != index && contains_any(h, &["heure", "time"]) && !contains_any(h, &["timestamp"])
        })
        .map(|(_, h)| h.clone())
        .filter(|h| h != &name);
    (name, time)
}

pub fn inspect_text(text: &str) -> Result<Inspection, AppError> {
    let delimiter = detect_delimiter(text);
    let all = rows(text, delimiter)?;
    if all.len() < 2 {
        return Err(AppError::Validation(
            "fichier CSV vide : il faut une ligne d'en-tête et au moins une ligne de données"
                .into(),
        ));
    }
    let skip_rows = header_row(&all);
    let header = all[skip_rows].clone();
    let data: Vec<Vec<String>> = all
        .iter()
        .skip(skip_rows + 1)
        .take(GUESS_ROWS)
        .cloned()
        .collect();
    let (date_name, time_name) = date_column(&header, &data);
    let date_index = header.iter().position(|h| h == &date_name).unwrap_or(0);
    let time_index = time_name
        .as_deref()
        .and_then(|t| header.iter().position(|h| h == t));
    let joined: Vec<String> = data
        .iter()
        .map(|r| {
            let d = r.get(date_index).cloned().unwrap_or_default();
            match time_index.and_then(|i| r.get(i)) {
                Some(t) => format!("{d} {t}"),
                None => d,
            }
        })
        .collect();
    let format = dates::guess(joined.iter().map(String::as_str)).unwrap_or(dates::ISO);
    let preset = libreview(&header);
    let columns = preset
        .clone()
        .unwrap_or_else(|| guess_columns(&header, &[date_name.as_str()]));
    let decimal = if data.iter().flatten().any(|c| {
        c.contains(',')
            && c.chars()
                .all(|ch| ch.is_ascii_digit() || ch == ',' || ch == '-')
    }) {
        ','
    } else {
        '.'
    };
    Ok(Inspection {
        signature: signature(&header),
        sample: all
            .iter()
            .skip(skip_rows + 1)
            .take(SAMPLE_ROWS)
            .cloned()
            .collect(),
        row_count: all.len() - skip_rows - 1,
        suggestion: CsvConfig {
            delimiter,
            decimal,
            skip_rows,
            date_column: date_name,
            time_column: time_name,
            date_format: format.to_string(),
            columns,
        },
        preset: preset.map(|_| "LibreView"),
        formats: dates::FORMATS.to_vec(),
        header,
    })
}

pub fn inspect_bytes(bytes: &[u8]) -> Result<Inspection, AppError> {
    inspect_text(&decode_text(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCALE: &str = "Date;Heure;Poids (kg);Masse grasse (%);IMC\n25/10/2026;07:30;72,4;18,5;23,1\n26/10/2026;07:41;72,1;18,4;23,0\n";

    #[test]
    fn a_scale_export_is_understood_without_any_help() {
        let i = inspect_text(SCALE).expect("inspection");
        let s = &i.suggestion;
        assert_eq!((s.delimiter, s.decimal, s.skip_rows), (';', ',', 0));
        assert_eq!(s.date_column, "Date");
        assert_eq!(s.time_column.as_deref(), Some("Heure"));
        assert_eq!(s.date_format, "%d/%m/%Y %H:%M");
        let metrics: Vec<&str> = s.columns.iter().map(|c| c.metric.as_str()).collect();
        assert_eq!(metrics, vec!["weight", "fat_ratio", "bmi"]);
        assert_eq!(s.columns[0].unit, "kg");
        assert_eq!(i.row_count, 2);
        assert_eq!(i.sample.len(), 2);
        assert!(i.preset.is_none());
    }

    #[test]
    fn a_libreview_export_is_recognised_with_its_metadata_line() {
        let text = "Patient report,Imprime le 2026-10-25,,,,,\nDevice,Serial Number,Device Timestamp,Record Type,Historic Glucose mg/dL,Scan Glucose mg/dL,Strip Glucose mg/dL\nFreeStyle,ABC,25-10-2026 07:30,0,99,,\nFreeStyle,ABC,25-10-2026 07:45,1,,110,\n";
        let i = inspect_text(text).expect("inspection");
        assert_eq!(i.preset, Some("LibreView"));
        assert_eq!(i.suggestion.skip_rows, 1);
        assert_eq!(i.suggestion.date_column, "Device Timestamp");
        assert_eq!(i.suggestion.date_format, "%d-%m-%Y %H:%M");
        assert_eq!(i.suggestion.columns.len(), 3);
        assert!(i.suggestion.columns.iter().all(|c| c.unit == "mg/dL"));
    }

    #[test]
    fn a_blood_pressure_log_pairs_its_two_columns() {
        let text = "Date,Systolic (mmHg),Diastolic (mmHg),Pulse\n2026-10-25 07:30,121,79,64\n";
        let i = inspect_text(text).expect("inspection");
        let metrics: Vec<&str> = i
            .suggestion
            .columns
            .iter()
            .map(|c| c.metric.as_str())
            .collect();
        assert_eq!(metrics, vec!["systolic", "diastolic"]);
        assert_eq!(i.suggestion.date_format, "iso");
        assert_eq!(i.suggestion.decimal, '.');
    }

    #[test]
    fn a_file_without_data_says_so() {
        let err = inspect_text("Date;Poids\n").unwrap_err();
        assert!(err.to_string().contains("vide"), "{err}");
        assert!(inspect_text("").is_err());
    }
}
