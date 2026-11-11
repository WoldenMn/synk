use crate::sources::{
    Domain, RevokeContext, RevokeFuture, Source, SourceId, SourceKind, SyncContext, SyncFuture,
};

const DOMAINS: &[Domain] = &[
    Domain::Sleep,
    Domain::Activity,
    Domain::Workouts,
    Domain::Vitals,
];

pub struct GoogleHealthSource {
    base: Option<String>,
}

pub static GOOGLE_HEALTH: GoogleHealthSource = GoogleHealthSource::new();

impl GoogleHealthSource {
    pub const fn new() -> Self {
        Self { base: None }
    }

    pub fn at(base: impl Into<String>) -> Self {
        Self {
            base: Some(base.into()),
        }
    }
}

impl Default for GoogleHealthSource {
    fn default() -> Self {
        Self::new()
    }
}

impl Source for GoogleHealthSource {
    fn id(&self) -> SourceId {
        SourceId::GOOGLE_HEALTH
    }

    fn label(&self) -> &'static str {
        "Google Health"
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

    fn sync<'a>(&'a self, ctx: &'a SyncContext<'a>) -> SyncFuture<'a> {
        Box::pin(async move {
            let (token_base, api_base) = match self.base.as_deref() {
                Some(base) => (base, base),
                None => (super::spec::TOKEN_BASE, super::spec::HEALTH_BASE),
            };
            super::sync::run(token_base, api_base, ctx).await
        })
    }

    fn revoke<'a>(&'a self, ctx: &'a RevokeContext<'a>) -> RevokeFuture<'a> {
        Box::pin(async move {
            let token_base = self.base.as_deref().unwrap_or(super::spec::TOKEN_BASE);
            super::revoke::run(token_base, ctx).await
        })
    }
}
