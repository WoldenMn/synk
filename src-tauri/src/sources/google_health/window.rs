use chrono::{DateTime, Duration, NaiveDate, Utc};

pub const INITIAL_DAYS: i64 = 180;
pub const OVERLAP_DAYS: i64 = 7;
pub const WIDE_ROLLUP_DAYS: i64 = 90;
pub const NARROW_ROLLUP_DAYS: i64 = 14;

pub fn window(last_sync: i64, now: DateTime<Utc>) -> (NaiveDate, NaiveDate) {
    let today = now.date_naive();
    let start = match DateTime::from_timestamp(last_sync, 0) {
        Some(last) if last_sync > 0 => last.date_naive() - Duration::days(OVERLAP_DAYS),
        _ => today - Duration::days(INITIAL_DAYS),
    };
    (start, today + Duration::days(1))
}

pub fn chunks(start: NaiveDate, end: NaiveDate, max_days: i64) -> Vec<(NaiveDate, NaiveDate)> {
    let mut out = Vec::new();
    let mut from = start;
    while from < end {
        let to = (from + Duration::days(max_days)).min(end);
        out.push((from, to));
        from = to;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at(y: i32, m: u32, d: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, m, d, 9, 30, 0).single().unwrap()
    }

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn a_first_sync_goes_back_the_initial_window_and_ends_tomorrow_exclusive() {
        let (start, end) = window(0, at(2026, 10, 26));
        assert_eq!(start, date(2026, 4, 29));
        assert_eq!(end, date(2026, 10, 27));
    }

    #[test]
    fn an_incremental_sync_restarts_a_week_before_the_last_one() {
        let (start, end) = window(at(2026, 10, 20).timestamp(), at(2026, 10, 26));
        assert_eq!(start, date(2026, 10, 13));
        assert_eq!(end, date(2026, 10, 27));
    }

    #[test]
    fn a_range_is_cut_in_closed_open_chunks_that_never_exceed_the_limit() {
        let parts = chunks(date(2026, 1, 1), date(2026, 2, 6), 14);
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], (date(2026, 1, 1), date(2026, 1, 15)));
        assert_eq!(parts[1].0, parts[0].1);
        assert_eq!(parts[2], (date(2026, 1, 29), date(2026, 2, 6)));
        assert!(parts.iter().all(|(a, b)| (*b - *a).num_days() <= 14));
        assert!(chunks(date(2026, 1, 1), date(2026, 1, 1), 14).is_empty());
    }

    #[test]
    fn a_first_sync_needs_three_wide_chunks_and_thirteen_narrow_ones() {
        let (start, end) = window(0, at(2026, 10, 26));
        assert_eq!(chunks(start, end, WIDE_ROLLUP_DAYS).len(), 3);
        assert_eq!(chunks(start, end, NARROW_ROLLUP_DAYS).len(), 13);
    }
}
