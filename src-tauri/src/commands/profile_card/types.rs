use serde::Serialize;

#[derive(Debug, Default, Serialize)]
pub struct Body {
    pub age: Option<i64>,
    pub sex: Option<String>,
    pub height_cm: Option<f64>,
    pub weight_kg: Option<f64>,
    pub bmi: Option<f64>,
    pub fat_ratio: Option<f64>,
    pub muscle_mass: Option<f64>,
    pub hydration: Option<f64>,
    pub measured_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct KeyMeasurement {
    pub slug: String,
    pub label: String,
    pub value: Option<f64>,
    pub measured_at: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct Health {
    pub active_treatments: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SizeEntry {
    pub category: String,
    pub value: String,
}

#[derive(Debug, Serialize)]
pub struct Record {
    pub key: String,
    pub label: String,
    pub value: f64,
    pub unit: String,
    pub achieved_at: String,
}

#[derive(Debug, Serialize)]
pub struct ProfileCard {
    pub body: Body,
    pub key_measurements: Vec<KeyMeasurement>,
    pub health: Health,
    pub sizes: Vec<SizeEntry>,
    pub records: Vec<Record>,
}
