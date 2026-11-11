use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceId(&'static str);

impl SourceId {
    pub const WITHINGS: SourceId = SourceId::new("withings");
    pub const OURA: SourceId = SourceId::new("oura");
    pub const POLAR: SourceId = SourceId::new("polar");
    pub const WHOOP: SourceId = SourceId::new("whoop");
    pub const GOOGLE_HEALTH: SourceId = SourceId::new("google_health");
    pub const APPLE_HEALTH: SourceId = SourceId::new("apple_health");
    pub const TRACE: SourceId = SourceId::new("trace");
    pub const FIT: SourceId = SourceId::new("fit");
    pub const CSV: SourceId = SourceId::new("csv");

    pub const fn new(id: &'static str) -> Self {
        Self(id)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Observation {
    pub metric: String,
    pub measured_at: DateTime<Utc>,
    pub value: f64,
    pub unit: String,
    pub source: SourceId,
    pub external_id: Option<String>,
    pub device: Option<String>,
}
