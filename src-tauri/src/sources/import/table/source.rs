use crate::error::AppError;
use crate::sources::{Domain, FileSpec, ImportContext, ImportFuture, Source, SourceId, SourceKind};

static FILE: FileSpec = FileSpec {
    extensions: &["csv", "txt"],
};

const DOMAINS: &[Domain] = &[
    Domain::Body,
    Domain::BloodPressure,
    Domain::Vitals,
    Domain::Glucose,
    Domain::Activity,
];

pub struct CsvSource;

pub static CSV: CsvSource = CsvSource;

impl Source for CsvSource {
    fn id(&self) -> SourceId {
        SourceId::CSV
    }

    fn label(&self) -> &'static str {
        "Fichiers CSV"
    }

    fn kind(&self) -> SourceKind {
        SourceKind::File(&FILE)
    }

    fn capabilities(&self) -> &'static [Domain] {
        DOMAINS
    }

    fn import<'a>(&'a self, _ctx: ImportContext<'a>) -> Option<ImportFuture<'a>> {
        Some(Box::pin(async {
            Err(AppError::Validation(
                "un fichier CSV passe par l'assistant de correspondance".into(),
            ))
        }))
    }
}
