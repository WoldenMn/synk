use crate::sources::{
    Domain, MetricDefinition, RevokeContext, RevokeFuture, Source, SourceId, SourceKind,
    SyncContext, SyncFuture,
};

const DOMAINS: &[Domain] = &[
    Domain::Sleep,
    Domain::Activity,
    Domain::Workouts,
    Domain::Vitals,
];

const DEFINITIONS: &[MetricDefinition] = &[
    MetricDefinition {
        slug: "hrv",
        label: "Variabilite cardiaque (VFC)",
        unit: "ms",
        category: "body_scan",
        sort_order: 82,
        decimals: 0,
        min_value: 1.0,
        max_value: 400.0,
    },
    MetricDefinition {
        slug: "readiness_score",
        label: "Score de preparation",
        unit: "pts",
        category: "body_scan",
        sort_order: 83,
        decimals: 0,
        min_value: 0.0,
        max_value: 100.0,
    },
    MetricDefinition {
        slug: "temperature_deviation",
        label: "Ecart de temperature cutanee",
        unit: "degC",
        category: "body_scan",
        sort_order: 84,
        decimals: 2,
        min_value: -10.0,
        max_value: 10.0,
    },
];

pub struct OuraSource {
    base: Option<String>,
}

pub static OURA: OuraSource = OuraSource::new();

impl OuraSource {
    pub const fn new() -> Self {
        Self { base: None }
    }

    pub fn at(base: impl Into<String>) -> Self {
        Self {
            base: Some(base.into()),
        }
    }
}

impl Default for OuraSource {
    fn default() -> Self {
        Self::new()
    }
}

impl Source for OuraSource {
    fn id(&self) -> SourceId {
        SourceId::OURA
    }

    fn label(&self) -> &'static str {
        "Oura"
    }

    fn kind(&self) -> SourceKind {
        SourceKind::OAuth(&super::spec::OAUTH)
    }

    fn capabilities(&self) -> &'static [Domain] {
        DOMAINS
    }

    fn steps(&self) -> u32 {
        4
    }

    fn developer_url(&self) -> Option<&'static str> {
        Some(super::spec::DEVELOPER_URL)
    }

    fn metric_definitions(&self) -> &'static [MetricDefinition] {
        DEFINITIONS
    }

    fn sync<'a>(&'a self, ctx: &'a SyncContext<'a>) -> SyncFuture<'a> {
        Box::pin(async move {
            let base = self.base.as_deref().unwrap_or(super::spec::API_BASE);
            super::sync::run(base, ctx).await
        })
    }

    fn revoke<'a>(&'a self, ctx: &'a RevokeContext<'a>) -> RevokeFuture<'a> {
        Box::pin(async move {
            let base = self.base.as_deref().unwrap_or(super::spec::API_BASE);
            super::revoke::run(base, ctx).await
        })
    }
}
