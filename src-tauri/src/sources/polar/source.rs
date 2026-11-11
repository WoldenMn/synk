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
        slug: "ans_charge",
        label: "Charge du systeme nerveux autonome (ANS)",
        unit: "pts",
        category: "body_scan",
        sort_order: 85,
        decimals: 1,
        min_value: -10.0,
        max_value: 10.0,
    },
    MetricDefinition {
        slug: "nightly_recharge_status",
        label: "Statut Nightly Recharge",
        unit: "pts",
        category: "body_scan",
        sort_order: 86,
        decimals: 0,
        min_value: 1.0,
        max_value: 6.0,
    },
];

pub struct PolarSource {
    base: Option<String>,
}

pub static POLAR: PolarSource = PolarSource::new();

impl PolarSource {
    pub const fn new() -> Self {
        Self { base: None }
    }

    pub fn at(base: impl Into<String>) -> Self {
        Self {
            base: Some(base.into()),
        }
    }
}

impl Default for PolarSource {
    fn default() -> Self {
        Self::new()
    }
}

impl Source for PolarSource {
    fn id(&self) -> SourceId {
        SourceId::POLAR
    }

    fn label(&self) -> &'static str {
        "Polar"
    }

    fn kind(&self) -> SourceKind {
        SourceKind::OAuth(&super::spec::OAUTH)
    }

    fn capabilities(&self) -> &'static [Domain] {
        DOMAINS
    }

    fn steps(&self) -> u32 {
        3
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
