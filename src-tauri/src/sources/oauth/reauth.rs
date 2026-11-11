pub const REFRESH_EXPIRY: &str = "refresh_expiry";
pub const NOTICE_LEAD_SECS: i64 = 86_400;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReauthStatus {
    Soon,
    Expired,
}

impl ReauthStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            ReauthStatus::Soon => "soon",
            ReauthStatus::Expired => "expired",
        }
    }
}

pub fn reauth_status(expires_at: i64, now: i64) -> Option<ReauthStatus> {
    if expires_at <= 0 || now < expires_at - NOTICE_LEAD_SECS {
        None
    } else if now >= expires_at {
        Some(ReauthStatus::Expired)
    } else {
        Some(ReauthStatus::Soon)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXPIRY: i64 = 1_800_000_000;

    #[test]
    fn nothing_is_announced_more_than_a_day_before_the_expiry() {
        assert_eq!(reauth_status(EXPIRY, EXPIRY - NOTICE_LEAD_SECS - 1), None);
        assert_eq!(reauth_status(EXPIRY, EXPIRY - 2 * NOTICE_LEAD_SECS), None);
    }

    #[test]
    fn the_notice_starts_exactly_one_day_before_the_expiry() {
        assert_eq!(
            reauth_status(EXPIRY, EXPIRY - NOTICE_LEAD_SECS),
            Some(ReauthStatus::Soon)
        );
        assert_eq!(reauth_status(EXPIRY, EXPIRY - 1), Some(ReauthStatus::Soon));
    }

    #[test]
    fn a_passed_expiry_is_reported_as_expired() {
        assert_eq!(reauth_status(EXPIRY, EXPIRY), Some(ReauthStatus::Expired));
        assert_eq!(
            reauth_status(EXPIRY, EXPIRY + 5 * NOTICE_LEAD_SECS),
            Some(ReauthStatus::Expired)
        );
    }

    #[test]
    fn an_unknown_expiry_never_triggers_a_notice() {
        assert_eq!(reauth_status(0, 1_900_000_000), None);
        assert_eq!(reauth_status(-1, 1_900_000_000), None);
    }
}
