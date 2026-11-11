use chrono::{DateTime, Duration, Utc};

pub const INITIAL_DAYS: i64 = 180;
pub const OVERLAP_DAYS: i64 = 7;
pub const CHUNK_DAYS: i64 = 27;
pub const MAX_BACK_DAYS: i64 = 364;

fn text(at: DateTime<Utc>) -> String {
    at.format("%Y-%m-%d").to_string()
}

pub fn activity_ranges(last_sync: i64, now: DateTime<Utc>) -> Vec<(String, String)> {
    let floor = now - Duration::days(MAX_BACK_DAYS);
    let start = match DateTime::from_timestamp(last_sync, 0) {
        Some(last) if last_sync > 0 => last - Duration::days(OVERLAP_DAYS),
        _ => now - Duration::days(INITIAL_DAYS),
    }
    .max(floor);
    let mut ranges = Vec::new();
    let mut from = start;
    loop {
        let to = (from + Duration::days(CHUNK_DAYS)).min(now);
        ranges.push((text(from), text(to)));
        if to >= now {
            return ranges;
        }
        from = to;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, TimeZone};

    fn at(y: i32, m: u32, d: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, m, d, 9, 30, 0).single().unwrap()
    }

    #[test]
    fn a_first_sync_walks_back_180_days_in_chunks_that_overlap_by_one_day() {
        let ranges = activity_ranges(0, at(2026, 10, 26));
        assert_eq!(ranges.len(), 7);
        assert_eq!(ranges[0].0, "2026-04-29");
        assert_eq!(ranges[0].1, "2026-05-26");
        assert_eq!(ranges[1].0, ranges[0].1);
        assert_eq!(ranges[6].1, "2026-10-26");
        for (from, to) in &ranges {
            let span =
                (to.parse::<NaiveDate>().unwrap() - from.parse::<NaiveDate>().unwrap()).num_days();
            assert!(span <= 27, "{from} {to}");
        }
    }

    #[test]
    fn an_incremental_sync_restarts_a_week_back_in_one_call() {
        let last = at(2026, 10, 20).timestamp();
        let ranges = activity_ranges(last, at(2026, 10, 26));
        assert_eq!(
            ranges,
            vec![("2026-10-13".to_string(), "2026-10-26".to_string())]
        );
    }

    #[test]
    fn a_very_old_cursor_never_asks_further_back_than_polar_allows() {
        let last = at(2024, 1, 1).timestamp();
        let ranges = activity_ranges(last, at(2026, 10, 26));
        assert_eq!(ranges[0].0, "2025-10-27");
    }
}
