use super::import_trace;
use crate::sources::{Domain, FileSpec, ImportContext, ImportFuture, Source, SourceId, SourceKind};

static FILE: FileSpec = FileSpec {
    extensions: &["gpx", "tcx"],
};

const DOMAINS: &[Domain] = &[Domain::Workouts];

pub struct TraceSource;

pub static TRACE: TraceSource = TraceSource;

impl Source for TraceSource {
    fn id(&self) -> SourceId {
        SourceId::TRACE
    }

    fn label(&self) -> &'static str {
        "Traces GPX et TCX"
    }

    fn kind(&self) -> SourceKind {
        SourceKind::File(&FILE)
    }

    fn capabilities(&self) -> &'static [Domain] {
        DOMAINS
    }

    fn import<'a>(&'a self, ctx: ImportContext<'a>) -> Option<ImportFuture<'a>> {
        Some(Box::pin(async move {
            import_trace(ctx.pool, ctx.path, ctx.cancel, ctx.on_progress).await
        }))
    }
}
