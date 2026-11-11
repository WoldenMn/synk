use crate::sources::{
    Domain, RevokeContext, RevokeFuture, Source, SourceId, SourceKind, SyncBatch, SyncContext,
    SyncFuture,
};

use super::client;
use super::sync::run_sync_all;
use super::types::SyncResult;

const DOMAINS: &[Domain] = &[
    Domain::Body,
    Domain::BloodPressure,
    Domain::Sleep,
    Domain::Activity,
    Domain::Workouts,
    Domain::Heart,
    Domain::Vitals,
    Domain::Devices,
];

pub struct WithingsSource {
    base: Option<String>,
}

pub static WITHINGS: WithingsSource = WithingsSource::new();

impl WithingsSource {
    pub const fn new() -> Self {
        Self { base: None }
    }

    pub fn at(base: impl Into<String>) -> Self {
        Self {
            base: Some(base.into()),
        }
    }
}

impl Default for WithingsSource {
    fn default() -> Self {
        Self::new()
    }
}

pub fn batch_from(result: SyncResult) -> SyncBatch {
    SyncBatch {
        observations: Vec::new(),
        written: vec![
            (Domain::Body, result.body_measures_count),
            (Domain::BloodPressure, result.blood_pressure_count),
            (Domain::Sleep, result.sleep_count),
            (Domain::Activity, result.activity_count),
            (Domain::Vitals, result.vitals_count),
            (Domain::Workouts, result.workouts_count),
            (Domain::Heart, result.heart_count),
            (Domain::Devices, result.devices_count),
        ],
        errors: result.errors,
    }
}

#[cfg(test)]
fn result_from(batch: &SyncBatch) -> SyncResult {
    SyncResult {
        body_measures_count: batch.written_in(Domain::Body),
        blood_pressure_count: batch.written_in(Domain::BloodPressure),
        sleep_count: batch.written_in(Domain::Sleep),
        activity_count: batch.written_in(Domain::Activity),
        vitals_count: batch.written_in(Domain::Vitals),
        workouts_count: batch.written_in(Domain::Workouts),
        heart_count: batch.written_in(Domain::Heart),
        devices_count: batch.written_in(Domain::Devices),
        errors: batch.errors.clone(),
    }
}

impl Source for WithingsSource {
    fn id(&self) -> SourceId {
        SourceId::WITHINGS
    }

    fn label(&self) -> &'static str {
        "Withings"
    }

    fn kind(&self) -> SourceKind {
        SourceKind::OAuth(&super::spec::OAUTH)
    }

    fn capabilities(&self) -> &'static [Domain] {
        DOMAINS
    }

    fn steps(&self) -> u32 {
        6
    }

    fn developer_url(&self) -> Option<&'static str> {
        Some("https://developer.withings.com/")
    }

    fn sync<'a>(&'a self, ctx: &'a SyncContext<'a>) -> SyncFuture<'a> {
        Box::pin(async move {
            let client = match &self.base {
                Some(base) => client::get_valid_client_at(base, ctx.pool, ctx.vault).await?,
                None => client::get_valid_client(ctx.pool, ctx.vault).await?,
            };
            let result = run_sync_all(ctx.pool, &client, ctx.on_progress).await;
            Ok(batch_from(result))
        })
    }

    fn revoke<'a>(&'a self, ctx: &'a RevokeContext<'a>) -> RevokeFuture<'a> {
        Box::pin(async move { super::revoke::run(self.base.as_deref(), ctx).await })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_result_survives_the_trip_through_a_batch() {
        let result = SyncResult {
            body_measures_count: 1,
            blood_pressure_count: 2,
            sleep_count: 3,
            activity_count: 4,
            vitals_count: 5,
            workouts_count: 6,
            heart_count: 7,
            devices_count: 8,
            errors: vec!["Coeur : 2555".into()],
        };
        let expected = format!("{result:?}");
        assert_eq!(format!("{:?}", result_from(&batch_from(result))), expected);
    }
}
