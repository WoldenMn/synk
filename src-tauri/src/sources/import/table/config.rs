use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::dates;
use super::targets::{self, Kind};
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ColumnMap {
    pub column: String,
    pub metric: String,
    pub unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CsvConfig {
    pub delimiter: char,
    pub decimal: char,
    pub skip_rows: usize,
    pub date_column: String,
    pub time_column: Option<String>,
    pub date_format: String,
    pub columns: Vec<ColumnMap>,
}

const MAX_SKIP_ROWS: usize = 10_000;
const MAX_COLUMNS: usize = 200;
const MAX_NAME_CHARS: usize = 200;

pub fn signature(header: &[String]) -> String {
    let mut hasher = Sha256::new();
    for name in header {
        let name = name.trim().to_lowercase();
        hasher.update((name.len() as u64).to_le_bytes());
        hasher.update(name.as_bytes());
    }
    hasher
        .finalize()
        .iter()
        .take(12)
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn invalid(text: impl Into<String>) -> AppError {
    AppError::Validation(text.into())
}

impl CsvConfig {
    pub fn validate_shape(&self) -> Result<(), AppError> {
        if !matches!(self.delimiter, ',' | ';' | '\t') {
            return Err(invalid(
                "séparateur de colonnes : virgule, point-virgule ou tabulation",
            ));
        }
        if !matches!(self.decimal, '.' | ',') {
            return Err(invalid("séparateur décimal : point ou virgule"));
        }
        if !dates::is_known(&self.date_format) {
            return Err(invalid(format!(
                "format de date inconnu : {}",
                self.date_format
            )));
        }
        if self.skip_rows > MAX_SKIP_ROWS {
            return Err(invalid("trop de lignes ignorées en tête de fichier"));
        }
        if self.columns.len() > MAX_COLUMNS {
            return Err(invalid("trop de colonnes associées"));
        }
        let names = [Some(&self.date_column), self.time_column.as_ref()]
            .into_iter()
            .flatten()
            .chain(self.columns.iter().map(|c| &c.column));
        for name in names {
            if name.chars().count() > MAX_NAME_CHARS {
                return Err(invalid("nom de colonne trop long"));
            }
        }
        if self.columns.is_empty() {
            return Err(invalid("aucune colonne n'est associée à une mesure"));
        }
        Ok(())
    }

    pub fn validate(&self, header: &[String]) -> Result<(), AppError> {
        self.validate_shape()?;
        let has = |name: &str| header.iter().any(|h| h == name);
        if !has(&self.date_column) {
            return Err(invalid(format!(
                "colonne de date introuvable : {}",
                self.date_column
            )));
        }
        if let Some(time) = &self.time_column {
            if !has(time) {
                return Err(invalid(format!("colonne d'heure introuvable : {time}")));
            }
        }
        for map in &self.columns {
            if !has(&map.column) {
                return Err(invalid(format!("colonne introuvable : {}", map.column)));
            }
            let target = targets::find(&map.metric)
                .ok_or_else(|| invalid(format!("mesure inconnue : {}", map.metric)))?;
            if !target.units.iter().any(|u| u.id == map.unit) {
                return Err(invalid(format!(
                    "unité {} impossible pour {}",
                    map.unit, target.label
                )));
            }
        }
        let count = |kind: Kind| {
            self.columns
                .iter()
                .filter(|m| targets::find(&m.metric).is_some_and(|t| t.kind == kind))
                .count()
        };
        if count(Kind::Systolic) != count(Kind::Diastolic) || count(Kind::Systolic) > 1 {
            return Err(invalid(
                "la tension demande une colonne systolique et une colonne diastolique",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header() -> Vec<String> {
        ["Date", "Poids", "Sys", "Dia"].map(String::from).to_vec()
    }

    fn config() -> CsvConfig {
        CsvConfig {
            delimiter: ';',
            decimal: ',',
            skip_rows: 0,
            date_column: "Date".into(),
            time_column: None,
            date_format: "%d/%m/%Y".into(),
            columns: vec![ColumnMap {
                column: "Poids".into(),
                metric: "weight".into(),
                unit: "kg".into(),
            }],
        }
    }

    #[test]
    fn a_complete_configuration_is_valid() {
        assert!(config().validate(&header()).is_ok());
    }

    #[test]
    fn every_kind_of_mistake_gets_its_own_message() {
        let mut c = config();
        c.date_column = "Jour".into();
        assert!(c
            .validate(&header())
            .unwrap_err()
            .to_string()
            .contains("date introuvable"));
        let mut c = config();
        c.columns[0].unit = "mmHg".into();
        assert!(c
            .validate(&header())
            .unwrap_err()
            .to_string()
            .contains("impossible"));
        let mut c = config();
        c.columns[0].metric = "humeur".into();
        assert!(c
            .validate(&header())
            .unwrap_err()
            .to_string()
            .contains("mesure inconnue"));
        let mut c = config();
        c.columns.clear();
        assert!(c
            .validate(&header())
            .unwrap_err()
            .to_string()
            .contains("aucune colonne"));
        let mut c = config();
        c.date_format = "%Q".into();
        assert!(c
            .validate(&header())
            .unwrap_err()
            .to_string()
            .contains("format de date"));
        let mut c = config();
        c.columns.push(ColumnMap {
            column: "Sys".into(),
            metric: "systolic".into(),
            unit: "mmHg".into(),
        });
        assert!(c
            .validate(&header())
            .unwrap_err()
            .to_string()
            .contains("tension"));
    }

    #[test]
    fn the_signature_ignores_case_and_edges_but_not_the_columns() {
        let a = signature(&["Date".into(), " Poids ".into()]);
        let b = signature(&["date".into(), "poids".into()]);
        let c = signature(&["date".into(), "masse".into()]);
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(a.len(), 24);
    }
}
