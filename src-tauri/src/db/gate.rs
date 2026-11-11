use std::sync::atomic::{AtomicUsize, Ordering};

use crate::error::AppError;

static LONG_WRITES: AtomicUsize = AtomicUsize::new(0);

pub const BUSY_TIMEOUT_MS: u32 = 5000;
pub const FAST_BUSY_TIMEOUT_MS: u32 = 250;

pub const REFUSAL: &str =
    "Un import est en cours : cette opération reprendra possible à sa fin (ou en l'annulant).";

pub struct LongWrite(());

impl LongWrite {
    pub fn begin() -> Self {
        LONG_WRITES.fetch_add(1, Ordering::SeqCst);
        Self(())
    }
}

impl Drop for LongWrite {
    fn drop(&mut self) {
        LONG_WRITES.fetch_sub(1, Ordering::SeqCst);
    }
}

pub fn active() -> bool {
    LONG_WRITES.load(Ordering::SeqCst) > 0
}

pub fn busy_timeout_ms() -> u32 {
    if active() {
        FAST_BUSY_TIMEOUT_MS
    } else {
        BUSY_TIMEOUT_MS
    }
}

pub fn ensure_idle() -> Result<(), AppError> {
    if active() {
        return Err(AppError::Validation(REFUSAL.into()));
    }
    Ok(())
}

pub fn is_busy(e: &sqlx::Error) -> bool {
    e.as_database_error()
        .and_then(|d| d.code())
        .is_some_and(|c| matches!(c.as_ref(), "5" | "6" | "261" | "517" | "262" | "518"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_gate_is_open_until_a_long_write_begins_and_closes_again_when_it_ends() {
        let before = active();
        {
            let _guard = LongWrite::begin();
            assert!(active());
            assert_eq!(busy_timeout_ms(), FAST_BUSY_TIMEOUT_MS);
            assert!(ensure_idle().is_err());
        }
        assert_eq!(active(), before);
    }
}
