use chrono::{DateTime, Local, NaiveDateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stamp {
    pub utc: DateTime<Utc>,
    pub local: NaiveDateTime,
}

impl Stamp {
    pub fn on_this_computer(utc: DateTime<Utc>) -> Self {
        Self {
            utc,
            local: utc.with_timezone(&Local).naive_local(),
        }
    }

    pub fn from_unix(seconds: i64) -> Option<Self> {
        DateTime::from_timestamp(seconds, 0).map(Self::on_this_computer)
    }
}

pub fn local_day(stamp: &Stamp) -> String {
    stamp.local.format("%Y-%m-%d").to_string()
}

pub fn utc_text(at: DateTime<Utc>) -> String {
    at.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn an_instant_keeps_its_utc_text_and_gets_a_local_day() {
        let at = Utc.with_ymd_and_hms(2026, 10, 25, 12, 0, 0).unwrap();
        let stamp = Stamp::on_this_computer(at);
        assert_eq!(utc_text(stamp.utc), "2026-10-25T12:00:00Z");
        assert_eq!(local_day(&stamp), "2026-10-25");
    }

    #[test]
    fn a_unix_time_is_read_and_an_impossible_one_is_refused() {
        let stamp = Stamp::from_unix(1_792_929_600).expect("date");
        assert_eq!(utc_text(stamp.utc), "2026-10-25T12:00:00Z");
        assert!(Stamp::from_unix(i64::MAX).is_none());
    }
}
