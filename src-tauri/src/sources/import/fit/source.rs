use super::import_fit;
use crate::sources::{Domain, FileSpec, ImportContext, ImportFuture, Source, SourceId, SourceKind};

static FILE: FileSpec = FileSpec {
    extensions: &["fit"],
};

const DOMAINS: &[Domain] = &[
    Domain::Body,
    Domain::BloodPressure,
    Domain::Sleep,
    Domain::Workouts,
];

pub struct FitSource;

pub static FIT: FitSource = FitSource;

impl Source for FitSource {
    fn id(&self) -> SourceId {
        SourceId::FIT
    }

    fn label(&self) -> &'static str {
        "Fichiers FIT"
    }

    fn kind(&self) -> SourceKind {
        SourceKind::File(&FILE)
    }

    fn capabilities(&self) -> &'static [Domain] {
        DOMAINS
    }

    fn import<'a>(&'a self, ctx: ImportContext<'a>) -> Option<ImportFuture<'a>> {
        Some(Box::pin(async move {
            import_fit(ctx.pool, ctx.path, ctx.cancel, ctx.on_progress).await
        }))
    }
}
