use std::time::Duration;

pub const MINUTE_CAP: Duration = Duration::from_secs(60);
pub const DEFAULT_QUOTA_RESUME: Duration = Duration::from_secs(3600);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Throttle {
    Minute(Duration),
    Day(Duration),
}

fn whole_seconds(text: Option<&str>) -> Option<u64> {
    let value: f64 = text?.trim().parse().ok()?;
    (value.is_finite() && value >= 0.0).then(|| value.ceil() as u64)
}

fn limit_of_window(limit: &str, window: u64) -> Option<u64> {
    let tag = format!("window={window}");
    limit.split(',').skip(1).find_map(|entry| {
        let (count, rest) = entry.split_once(';')?;
        rest.trim()
            .split(';')
            .any(|part| part.trim() == tag)
            .then(|| count.trim().parse().ok())
            .flatten()
    })
}

fn closest_is_daily(limit: Option<&str>) -> bool {
    let Some(limit) = limit else {
        return false;
    };
    let closest: Option<u64> = limit.split(',').next().and_then(|n| n.trim().parse().ok());
    let day = limit_of_window(limit, 86_400);
    let minute = limit_of_window(limit, 60);
    closest.is_some() && closest == day && closest != minute
}

pub fn classify(limit: Option<&str>, reset: Option<&str>) -> Throttle {
    let reset = whole_seconds(reset);
    let beyond_a_minute = reset.is_some_and(|s| s > MINUTE_CAP.as_secs());
    if beyond_a_minute || closest_is_daily(limit) {
        return Throttle::Day(reset.map_or(DEFAULT_QUOTA_RESUME, Duration::from_secs));
    }
    let wait = reset.map_or(MINUTE_CAP, Duration::from_secs);
    Throttle::Minute(wait.clamp(Duration::from_secs(1), MINUTE_CAP))
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIMIT_MINUTE: &str = "100, 100;window=60, 10000;window=86400";
    const LIMIT_DAY: &str = "10000, 100;window=60, 10000;window=86400";

    #[test]
    fn a_reset_within_a_minute_is_the_minute_limit_and_is_waited_out() {
        assert_eq!(
            classify(Some(LIMIT_MINUTE), Some("3")),
            Throttle::Minute(Duration::from_secs(3))
        );
        assert_eq!(
            classify(Some(LIMIT_MINUTE), Some("59.2")),
            Throttle::Minute(Duration::from_secs(60))
        );
    }

    #[test]
    fn a_reset_beyond_a_minute_is_the_daily_quota() {
        assert_eq!(
            classify(Some(LIMIT_MINUTE), Some("43200")),
            Throttle::Day(Duration::from_secs(43200))
        );
    }

    #[test]
    fn a_closest_limit_equal_to_the_daily_one_is_the_daily_quota_even_with_a_small_reset() {
        assert_eq!(
            classify(Some(LIMIT_DAY), Some("30")),
            Throttle::Day(Duration::from_secs(30))
        );
    }

    #[test]
    fn missing_headers_are_a_minute_limit_of_a_full_minute() {
        assert_eq!(classify(None, None), Throttle::Minute(MINUTE_CAP));
        assert_eq!(
            classify(Some("n'importe quoi"), Some("abc")),
            Throttle::Minute(MINUTE_CAP)
        );
    }

    #[test]
    fn a_daily_quota_without_a_reset_waits_an_hour() {
        assert_eq!(
            classify(Some(LIMIT_DAY), None),
            Throttle::Day(DEFAULT_QUOTA_RESUME)
        );
    }

    #[test]
    fn a_zero_reset_still_waits_one_second() {
        assert_eq!(
            classify(Some(LIMIT_MINUTE), Some("0")),
            Throttle::Minute(Duration::from_secs(1))
        );
    }
}
