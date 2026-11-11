use serde::Deserialize;

use super::deserializers::{deserialize_heart_rate, deserialize_more};

#[derive(Debug, Deserialize)]
pub struct HeartListBody {
    #[serde(default)]
    pub series: Vec<HeartEntry>,
    #[serde(default, deserialize_with = "deserialize_more")]
    pub more: Option<i64>,
    #[allow(dead_code)]
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct HeartEntry {
    pub ecg: Option<HeartEcg>,
    // Champ deserialise depuis l'API `/heart/list` mais non utilise : les donnees
    // de tension arterielle sont consommees via le endpoint `/measure` (body_measures).
    // Conserve pour eviter un avertissement serde sur champ inconnu et rester
    // compatible avec les futures evolutions de l'API Withings.
    #[serde(rename = "bloodpressure")]
    #[allow(dead_code)]
    pub blood_pressure: Option<serde_json::Value>,
    #[serde(default, deserialize_with = "deserialize_heart_rate")]
    pub heart_rate: Option<HeartHr>,
    pub timestamp: i64,
}

#[derive(Debug, Deserialize)]
pub struct HeartEcg {
    pub signalid: Option<i64>,
    pub afib: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct HeartHr {
    pub value: Option<i64>,
    pub qrs_duration: Option<i64>,
}
