use std::future::Future;
use std::pin::Pin;
use std::time::{Duration, Instant};

pub type Sleeping<'a> = Pin<Box<dyn Future<Output = ()> + Send + 'a>>;

pub trait Clock: Send + Sync {
    fn now(&self) -> Duration;
    fn sleep(&self, wait: Duration) -> Sleeping<'_>;
}

pub struct SystemClock {
    origin: Instant,
}

impl SystemClock {
    pub fn new() -> Self {
        Self {
            origin: Instant::now(),
        }
    }
}

impl Default for SystemClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for SystemClock {
    fn now(&self) -> Duration {
        self.origin.elapsed()
    }

    fn sleep(&self, wait: Duration) -> Sleeping<'_> {
        Box::pin(tokio::time::sleep(wait))
    }
}
