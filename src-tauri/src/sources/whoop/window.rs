use chrono::{DateTime, Duration, Utc};

pub const INITIAL_DAYS: i64 = 180;
pub const OVERLAP_DAYS: i64 = 7;

pub fn window_start(last_sync: i64, now: DateTime<Utc>) -> String {
    let start = match DateTime::from_timestamp(last_sync, 0) {
        Some(last) if last_sync > 0 => last - Duration::days(OVERLAP_DAYS),
        _ => now - Duration::days(INITIAL_DAYS),
    };
    start.format("%Y-%m-%dT00:00:00.000Z").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at(y: i32, m: u32, d: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, m, d, 9, 30, 0).single().unwrap()
    }

    #[test]
    fn a_first_sync_goes_back_the_initial_window_from_midnight_utc() {
        assert_eq!(
            window_start(0, at(2026, 10, 26)),
            "2026-04-29T00:00:00.000Z"
        );
        assert_eq!(INITIAL_DAYS, 180);
    }

    #[test]
    fn an_incremental_sync_restarts_a_week_before_the_last_one() {
        let last = at(2026, 10, 20).timestamp();
        assert_eq!(
            window_start(last, at(2026, 10, 26)),
            "2026-10-13T00:00:00.000Z"
        );
        assert_eq!(OVERLAP_DAYS, 7);
    }
}
