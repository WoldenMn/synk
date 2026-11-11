use std::io::{self, BufRead, Read};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::error::AppError;

const GIB: u64 = 1024 * 1024 * 1024;
const MIB: u64 = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    pub bytes: u64,
    pub node: u64,
    pub depth: u32,
    pub elements: u64,
}

impl Limits {
    pub const APPLE_HEALTH: Limits = Limits {
        bytes: 16 * GIB,
        node: 4 * MIB,
        depth: 64,
        elements: 400_000_000,
    };

    pub const TRACE: Limits = Limits {
        bytes: GIB,
        node: 4 * MIB,
        depth: 64,
        elements: 20_000_000,
    };

    pub fn check_size(&self, bytes: u64, what: &str) -> Result<(), AppError> {
        if bytes > self.bytes {
            return Err(too_big(what));
        }
        Ok(())
    }
}

pub fn too_big(what: &str) -> AppError {
    AppError::Validation(format!(
        "{what} : volume au-delà de la limite acceptée, fichier refusé"
    ))
}

#[derive(Debug, Default)]
struct State {
    total: AtomicU64,
    mark: AtomicU64,
}

#[derive(Debug, Clone)]
pub struct Gauge(Arc<State>);

impl Gauge {
    pub fn event_done(&self) {
        self.0
            .mark
            .store(self.0.total.load(Ordering::Relaxed), Ordering::Relaxed);
    }
}

pub struct Guarded<R> {
    inner: R,
    state: Arc<State>,
    node: u64,
    tripped: bool,
}

impl<R: BufRead> Guarded<R> {
    pub fn new(inner: R, node: u64) -> (Self, Gauge) {
        let state = Arc::new(State::default());
        (
            Self {
                inner,
                state: Arc::clone(&state),
                node,
                tripped: false,
            },
            Gauge(state),
        )
    }
}

impl<R: BufRead> Read for Guarded<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = {
            let available = self.fill_buf()?;
            let n = available.len().min(buf.len());
            buf[..n].copy_from_slice(&available[..n]);
            n
        };
        self.consume(n);
        Ok(n)
    }
}

impl<R: BufRead> BufRead for Guarded<R> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        if self.tripped {
            return Err(io::Error::other("nœud XML trop volumineux, fichier refusé"));
        }
        self.inner.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.inner.consume(amt);
        let total = self.state.total.fetch_add(amt as u64, Ordering::Relaxed) + amt as u64;
        let mark = self.state.mark.load(Ordering::Relaxed);
        if total.saturating_sub(mark) > self.node {
            self.tripped = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn a_node_larger_than_the_budget_is_refused_while_events_reset_the_count() {
        let (mut guarded, gauge) = Guarded::new(Cursor::new(vec![b'a'; 100]), 40);
        let mut chunk = [0u8; 30];
        assert_eq!(guarded.read(&mut chunk).unwrap(), 30);
        gauge.event_done();
        assert_eq!(guarded.read(&mut chunk).unwrap(), 30);
        gauge.event_done();
        assert_eq!(guarded.read(&mut chunk).unwrap(), 30);
        assert_eq!(guarded.read(&mut chunk).unwrap(), 10);
        gauge.event_done();

        let (mut guarded, _) = Guarded::new(Cursor::new(vec![b'a'; 100]), 40);
        let mut big = [0u8; 50];
        assert_eq!(guarded.read(&mut big).unwrap(), 50);
        let err = guarded.read(&mut big).unwrap_err();
        assert!(err.to_string().contains("volumineux"), "{err}");
    }

    #[test]
    fn size_checks_name_the_file_kind() {
        let limits = Limits {
            bytes: 10,
            ..Limits::TRACE
        };
        assert!(limits.check_size(10, "GPX").is_ok());
        let err = limits.check_size(11, "GPX").unwrap_err();
        assert!(err.to_string().contains("GPX"), "{err}");
    }
}
