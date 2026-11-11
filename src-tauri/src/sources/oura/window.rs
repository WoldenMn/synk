use chrono::{DateTime, Duration, Utc};

pub const INITIAL_DAYS: i64 = 180;
pub const OVERLAP_DAYS: i64 = 7;

pub fn window(last_sync: i64, now: DateTime<Utc>) -> (String, String) {
    let start = match DateTime::from_timestamp(last_sync, 0) {
        Some(last) if last_sync > 0 => last - Duration::days(OVERLAP_DAYS),
        _ => now - Duration::days(INITIAL_DAYS),
    };
    let end = now + Duration::days(1);
    (
        start.format("%Y-%m-%d").to_string(),
        end.format("%Y-%m-%d").to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at(y: i32, m: u32, d: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, m, d, 9, 30, 0).single().unwrap()
    }

    #[test]
    fn a_first_sync_goes_back_the_initial_window_and_ends_tomorrow() {
        let (start, end) = window(0, at(2026, 10, 26));
        assert_eq!(start, "2026-04-29");
        assert_eq!(end, "2026-10-27");
        assert_eq!(INITIAL_DAYS, 180);
    }

    #[test]
    fn an_incremental_sync_restarts_a_week_before_the_last_one() {
        let last = at(2026, 10, 20).timestamp();
        let (start, end) = window(last, at(2026, 10, 26));
        assert_eq!(start, "2026-10-13");
        assert_eq!(end, "2026-10-27");
        assert_eq!(OVERLAP_DAYS, 7);
    }
}
