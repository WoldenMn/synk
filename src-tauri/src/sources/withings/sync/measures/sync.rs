use std::collections::HashMap;

use sqlx::SqlitePool;

use crate::sources::withings::client::WithingsClient;
use crate::sources::withings::mapping;
use crate::sources::withings::types::*;

use super::super::metrics::{preload_def_ids, upsert_pulse_alone, DefMeta};
use super::super::{get_last_sync, update_last_sync};
use super::body::upsert_body_group;
use super::bp::upsert_bp_group;
use super::vitals::upsert_vital_group;
use crate::error::AppError;

#[derive(Clone, Copy)]
pub(super) enum MeasureCategory {
    Body,
    BloodPressure,
    Vital,
}

pub async fn sync_body_measures(
    client: &WithingsClient,
    pool: &SqlitePool,
) -> Result<(usize, usize, usize), AppError> {
    let last_sync = get_last_sync(pool, "measures").await?;
    let cursor = chrono::Utc::now().timestamp();

    // Préchargé une fois (anti N+1) pour le dual-write `metrics` intra-transaction.
    let def_id_map = preload_def_ids(pool).await?;

    let body_meastypes = mapping::join_meastypes(mapping::BODY_TYPES);
    let bp_meastypes = mapping::join_meastypes(mapping::BP_TYPES);
    let vital_meastypes = mapping::join_meastypes(mapping::VITAL_TYPES);

    let mut body_count = 0usize;
    let mut bp_count = 0usize;
    let mut vitals_count = 0usize;

    let last_update_str = last_sync.to_string();
    body_count += fetch_and_upsert_measures(
        client,
        pool,
        &body_meastypes,
        &last_update_str,
        MeasureCategory::Body,
        &def_id_map,
    )
    .await?;

    bp_count += fetch_and_upsert_measures(
        client,
        pool,
        &bp_meastypes,
        &last_update_str,
        MeasureCategory::BloodPressure,
        &def_id_map,
    )
    .await?;

    vitals_count += fetch_and_upsert_measures(
        client,
        pool,
        &vital_meastypes,
        &last_update_str,
        MeasureCategory::Vital,
        &def_id_map,
    )
    .await?;

    update_last_sync(pool, "measures", cursor).await?;

    Ok((body_count, bp_count, vitals_count))
}

async fn fetch_and_upsert_measures(
    client: &WithingsClient,
    pool: &SqlitePool,
    meastypes: &str,
    lastupdate: &str,
    category: MeasureCategory,
    def_id_map: &HashMap<i64, DefMeta>,
) -> Result<usize, AppError> {
    // Cluster A (#8/#9) : collecter TOUTES les pages reseau AVANT d'ouvrir la
    // transaction, sinon le write-lock WAL est tenu pendant les appels HTTP
    // (deadlock du refresh OAuth, "database is locked" sur les ecritures user).
    let mut groups = Vec::new();
    let mut offset: Option<String> = None;

    loop {
        let mut params = vec![
            ("action", "getmeas"),
            ("meastypes", meastypes),
            ("lastupdate", lastupdate),
        ];
        let offset_str;
        if let Some(ref o) = offset {
            offset_str = o.clone();
            params.push(("offset", &offset_str));
        }

        let resp: WithingsResponse<MeasureBody> = client.post("/measure", &params).await?;

        let more = resp.body.more;
        let next_offset = resp.body.offset;
        groups.extend(resp.body.measuregrps);

        match (more, next_offset) {
            (Some(1), Some(o)) => offset = Some(o.to_string()),
            _ => break,
        }
    }

    let mut total = 0usize;
    let mut tx = pool.begin().await?;
    for grp in &groups {
        total += persist_group(&mut tx, grp, category, def_id_map).await?;
    }
    tx.commit().await?;

    Ok(total)
}

pub(super) async fn persist_group(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    grp: &MeasureGroup,
    category: MeasureCategory,
    def_id_map: &HashMap<i64, DefMeta>,
) -> Result<usize, AppError> {
    match category {
        MeasureCategory::Body => upsert_body_group(&mut *tx, grp, def_id_map).await,
        MeasureCategory::Vital => upsert_vital_group(&mut *tx, grp, def_id_map).await,
        MeasureCategory::BloodPressure => {
            let written = upsert_bp_group(&mut *tx, grp).await?;
            if written == 0 {
                upsert_pulse_alone(&mut *tx, grp, def_id_map).await?;
            }
            Ok(written)
        }
    }
}
