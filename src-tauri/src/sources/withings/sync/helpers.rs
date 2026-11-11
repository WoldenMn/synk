use crate::error::AppError;

/// Fenetre de fetch incrementale en YYYY-MM-DD : (start_date, end_date).
/// `last_sync == 0` -> backfill de BACKFILL_DAYS jours ; sinon last_sync - 1 jour
/// (rollback d'1 jour pour rattraper les mesures du jour de la derniere sync).
pub const BACKFILL_DAYS: i64 = 180;

pub fn measured_at_iso(date: i64) -> Option<String> {
    chrono::DateTime::from_timestamp(date, 0).map(|d| d.format("%Y-%m-%dT%H:%M:%SZ").to_string())
}

pub fn compute_ymd_range(last_sync: i64) -> (String, String) {
    let now = chrono::Utc::now();
    let start = if last_sync == 0 {
        now.checked_sub_signed(chrono::Duration::days(BACKFILL_DAYS))
            .unwrap_or(now)
            .format("%Y-%m-%d")
            .to_string()
    } else {
        chrono::DateTime::from_timestamp(last_sync, 0)
            .map(|d| {
                (d - chrono::Duration::days(1))
                    .format("%Y-%m-%d")
                    .to_string()
            })
            .unwrap_or_else(|| now.format("%Y-%m-%d").to_string())
    };
    (start, now.format("%Y-%m-%d").to_string())
}

/// Collecte toutes les pages d'un endpoint Withings pagine par "batch-size"
/// (offset += nombre d'elements de la page) AVANT toute transaction (invariant
/// Cluster A : ne jamais tenir le write-lock WAL pendant les appels HTTP).
/// `fetch_page(offset)` renvoie (items_de_la_page, more). S'arrete quand
/// `more != Some(1)` ou que la page est vide (garde anti-boucle-infinie).
pub async fn paginate_batch<T, F, Fut>(mut fetch_page: F) -> Result<Vec<T>, AppError>
where
    F: FnMut(i64) -> Fut,
    Fut: std::future::Future<Output = Result<(Vec<T>, Option<i64>), AppError>>,
{
    let mut all = Vec::new();
    let mut offset = 0i64;
    loop {
        let (items, more) = fetch_page(offset).await?;
        let batch = items.len();
        all.extend(items);
        if more == Some(1) && batch > 0 {
            offset += batch as i64;
        } else {
            break;
        }
    }
    Ok(all)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn paginate_batch_collects_until_more_zero() {
        let r: Vec<i64> = paginate_batch(|offset| async move {
            match offset {
                0 => Ok::<_, crate::error::AppError>((vec![1, 2], Some(1))),
                2 => Ok((vec![3], Some(0))),
                _ => Ok((vec![], None)),
            }
        })
        .await
        .unwrap();
        assert_eq!(r, vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn paginate_batch_stops_on_empty_batch_guard() {
        // more=Some(1) mais page vide => stop (garde anti-boucle-infinie).
        let r: Vec<i64> = paginate_batch(|offset| async move {
            if offset == 0 {
                Ok::<_, crate::error::AppError>((vec![10], Some(1)))
            } else {
                Ok((vec![], Some(1)))
            }
        })
        .await
        .unwrap();
        assert_eq!(r, vec![10]);
    }

    #[test]
    fn a_timestamp_out_of_range_gives_no_date() {
        assert_eq!(measured_at_iso(i64::MAX), None);
        assert_eq!(measured_at_iso(0).as_deref(), Some("1970-01-01T00:00:00Z"));
    }

    #[test]
    fn backfill_when_never_synced() {
        let (start, end) = compute_ymd_range(0);
        assert_eq!(start.len(), 10);
        assert_eq!(end.len(), 10);
        assert!(start < end, "start (backfill 180j) doit preceder end");
    }

    #[test]
    fn incremental_uses_last_sync_minus_one_day() {
        // 2023-11-15 00:00:00 UTC = 1700006400
        let (start, _) = compute_ymd_range(1_700_006_400);
        assert_eq!(start, "2023-11-14", "rollback d'1 jour");
    }
}
