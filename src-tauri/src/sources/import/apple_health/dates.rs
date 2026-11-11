use chrono::{DateTime, FixedOffset, Utc};

pub use crate::sources::import::time::{local_day, utc_text, Stamp};

const FORMAT: &str = "%Y-%m-%d %H:%M:%S %z";

pub fn parse(text: &str) -> Option<Stamp> {
    let parsed: DateTime<FixedOffset> = DateTime::parse_from_str(text, FORMAT).ok()?;
    Some(Stamp {
        utc: parsed.with_timezone(&Utc),
        local: parsed.naive_local(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_apple_date_keeps_its_instant_and_its_local_clock() {
        let s = parse("2026-10-25 23:30:00 +0200").expect("date");
        assert_eq!(utc_text(s.utc), "2026-10-25T21:30:00Z");
        assert_eq!(local_day(&s), "2026-10-25");
        assert_eq!(s.local.format("%H:%M").to_string(), "23:30");
    }

    #[test]
    fn a_local_evening_can_fall_on_the_next_utc_day() {
        let s = parse("2026-10-25 23:30:00 -0500").expect("date");
        assert_eq!(utc_text(s.utc), "2026-10-26T04:30:00Z");
        assert_eq!(local_day(&s), "2026-10-25");
    }

    #[test]
    fn a_half_hour_offset_and_garbage_are_handled() {
        let s = parse("2026-01-02 03:04:05 +0530").expect("date");
        assert_eq!(utc_text(s.utc), "2026-01-01T21:34:05Z");
        assert!(parse("2026-10-25").is_none());
        assert!(parse("").is_none());
        assert!(parse("not a date").is_none());
    }
}
