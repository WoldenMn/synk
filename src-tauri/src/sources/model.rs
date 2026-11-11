use super::oauth::OAuthSpec;
pub use crate::metric_model::{Observation, SourceId};

#[derive(Debug, Clone, Copy)]
pub enum SourceKind {
    OAuth(&'static OAuthSpec),
    File(&'static FileSpec),
}

#[derive(Debug, PartialEq, Eq)]
pub struct FileSpec {
    pub extensions: &'static [&'static str],
}

impl PartialEq for SourceKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (SourceKind::OAuth(a), SourceKind::OAuth(b)) => a.id == b.id,
            (SourceKind::File(a), SourceKind::File(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for SourceKind {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Domain {
    Body,
    BloodPressure,
    Sleep,
    Activity,
    Workouts,
    Heart,
    Vitals,
    Glucose,
    Devices,
}

impl Domain {
    pub const fn as_str(self) -> &'static str {
        match self {
            Domain::Body => "body",
            Domain::BloodPressure => "blood_pressure",
            Domain::Sleep => "sleep",
            Domain::Activity => "activity",
            Domain::Workouts => "workouts",
            Domain::Heart => "heart",
            Domain::Vitals => "vitals",
            Domain::Glucose => "glucose",
            Domain::Devices => "devices",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MetricDefinition {
    pub slug: &'static str,
    pub label: &'static str,
    pub unit: &'static str,
    pub category: &'static str,
    pub sort_order: i64,
    pub decimals: i64,
    pub min_value: f64,
    pub max_value: f64,
}

#[derive(Debug, Default)]
pub struct SyncBatch {
    pub observations: Vec<Observation>,
    pub written: Vec<(Domain, usize)>,
    pub errors: Vec<String>,
}

impl SyncBatch {
    pub fn written_in(&self, domain: Domain) -> usize {
        self.written
            .iter()
            .filter(|(d, _)| *d == domain)
            .map(|(_, n)| n)
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn written_in_adds_up_the_counts_of_one_domain_only() {
        let batch = SyncBatch {
            written: vec![(Domain::Sleep, 2), (Domain::Body, 5), (Domain::Sleep, 3)],
            ..SyncBatch::default()
        };
        assert_eq!(batch.written_in(Domain::Sleep), 5);
        assert_eq!(batch.written_in(Domain::Body), 5);
        assert_eq!(batch.written_in(Domain::Heart), 0);
    }

    #[test]
    fn two_oauth_kinds_are_equal_when_they_share_a_source_id() {
        use crate::sources::oauth::token::standard_json;
        use crate::sources::oauth::ClientAuth;

        static A: OAuthSpec = OAuthSpec {
            id: SourceId::new("x"),
            authorize_url: "",
            api_base: "",
            token_path: "",
            scopes: "",
            pkce: false,
            redirect_path: None,
            token_params: &[],
            authorize_params: &[],
            refresh_lifetime_secs: None,
            parse_token: standard_json,
            client_auth: ClientAuth::Body,
        };
        static B: OAuthSpec = OAuthSpec {
            id: SourceId::new("y"),
            authorize_url: "",
            api_base: "",
            token_path: "",
            scopes: "",
            pkce: true,
            redirect_path: None,
            token_params: &[],
            authorize_params: &[],
            refresh_lifetime_secs: None,
            parse_token: standard_json,
            client_auth: ClientAuth::Body,
        };
        assert_eq!(SourceKind::OAuth(&A), SourceKind::OAuth(&A));
        assert_ne!(SourceKind::OAuth(&A), SourceKind::OAuth(&B));
        static F: FileSpec = FileSpec {
            extensions: &["zip"],
        };
        static G: FileSpec = FileSpec {
            extensions: &["csv"],
        };
        assert_ne!(SourceKind::OAuth(&A), SourceKind::File(&F));
        assert_eq!(SourceKind::File(&F), SourceKind::File(&F));
        assert_ne!(SourceKind::File(&F), SourceKind::File(&G));
    }

    #[test]
    fn a_source_id_names_itself() {
        assert_eq!(SourceId::WITHINGS.as_str(), "withings");
        assert_ne!(SourceId::WITHINGS, SourceId::OURA);
        assert_eq!(SourceId::OURA.as_str(), "oura");
    }
}
