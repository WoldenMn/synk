use super::run::import_apple_health;
use crate::sources::{
    Domain, FileSpec, ImportContext, ImportFuture, MetricDefinition, Source, SourceId, SourceKind,
};

static FILE: FileSpec = FileSpec {
    extensions: &["zip", "xml"],
};

const DOMAINS: &[Domain] = &[
    Domain::Body,
    Domain::BloodPressure,
    Domain::Sleep,
    Domain::Activity,
    Domain::Workouts,
    Domain::Vitals,
    Domain::Glucose,
];

const DEFINITIONS: &[MetricDefinition] = &[
    MetricDefinition {
        slug: "hrv_sdnn",
        label: "Variabilite cardiaque (SDNN)",
        unit: "ms",
        category: "body_scan",
        sort_order: 90,
        decimals: 0,
        min_value: 1.0,
        max_value: 500.0,
    },
    MetricDefinition {
        slug: "blood_glucose",
        label: "Glycemie",
        unit: "mmol/L",
        category: "body_scan",
        sort_order: 91,
        decimals: 1,
        min_value: 1.0,
        max_value: 40.0,
    },
];

pub struct AppleHealthSource;

pub static APPLE_HEALTH: AppleHealthSource = AppleHealthSource;

impl Source for AppleHealthSource {
    fn id(&self) -> SourceId {
        SourceId::APPLE_HEALTH
    }

    fn label(&self) -> &'static str {
        "Apple Santé"
    }

    fn kind(&self) -> SourceKind {
        SourceKind::File(&FILE)
    }

    fn capabilities(&self) -> &'static [Domain] {
        DOMAINS
    }

    fn metric_definitions(&self) -> &'static [MetricDefinition] {
        DEFINITIONS
    }

    fn import<'a>(&'a self, ctx: ImportContext<'a>) -> Option<ImportFuture<'a>> {
        Some(Box::pin(async move {
            import_apple_health(ctx.pool, ctx.path, ctx.cancel, ctx.on_progress).await
        }))
    }
}
