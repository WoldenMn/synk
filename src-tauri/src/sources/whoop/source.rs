use std::sync::Arc;

use crate::sources::oauth::clock::{Clock, SystemClock};
use crate::sources::oauth::pacer::PER_MINUTE;
use crate::sources::{
    Domain, MetricDefinition, Source, SourceId, SourceKind, SyncContext, SyncFuture,
};

const DOMAINS: &[Domain] = &[
    Domain::Sleep,
    Domain::Activity,
    Domain::Workouts,
    Domain::Vitals,
];

const DEFINITIONS: &[MetricDefinition] = &[
    MetricDefinition {
        slug: "recovery_score",
        label: "Score de recuperation",
        unit: "%",
        category: "body_scan",
        sort_order: 87,
        decimals: 0,
        min_value: 0.0,
        max_value: 100.0,
    },
    MetricDefinition {
        slug: "resting_heart_rate",
        label: "Frequence cardiaque au repos",
        unit: "bpm",
        category: "body_scan",
        sort_order: 88,
        decimals: 0,
        min_value: 20.0,
        max_value: 200.0,
    },
    MetricDefinition {
        slug: "strain",
        label: "Effort du jour (strain)",
        unit: "pts",
        category: "body_scan",
        sort_order: 89,
        decimals: 1,
        min_value: 0.0,
        max_value: 21.0,
    },
];

pub struct WhoopSource {
    base: Option<String>,
    per_minute: usize,
    clock: Option<Arc<dyn Clock>>,
}

pub static WHOOP: WhoopSource = WhoopSource::new();

impl WhoopSource {
    pub const fn new() -> Self {
        Self {
            base: None,
            per_minute: PER_MINUTE,
            clock: None,
        }
    }

    pub fn at(base: impl Into<String>) -> Self {
        Self {
            base: Some(base.into()),
            per_minute: PER_MINUTE,
            clock: None,
        }
    }

    pub fn at_paced(base: impl Into<String>, per_minute: usize, clock: Arc<dyn Clock>) -> Self {
        Self {
            base: Some(base.into()),
            per_minute,
            clock: Some(clock),
        }
    }
}

impl Default for WhoopSource {
    fn default() -> Self {
        Self::new()
    }
}

impl Source for WhoopSource {
    fn id(&self) -> SourceId {
        SourceId::WHOOP
    }

    fn label(&self) -> &'static str {
        "Whoop"
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
            let clock = self
                .clock
                .clone()
                .unwrap_or_else(|| Arc::new(SystemClock::new()));
            super::sync::run(base, ctx, self.per_minute, clock).await
        })
    }

    fn revoke<'a>(
        &'a self,
        ctx: &'a crate::sources::RevokeContext<'a>,
    ) -> crate::sources::RevokeFuture<'a> {
        Box::pin(async move {
            let base = self.base.as_deref().unwrap_or(super::spec::API_BASE);
            super::revoke::run(base, ctx).await
        })
    }
}
