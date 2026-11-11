use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;

use super::clock::Clock;

pub const PER_MINUTE: usize = 90;
pub const WINDOW: Duration = Duration::from_secs(60);

pub struct Pacer {
    clock: Arc<dyn Clock>,
    per_minute: usize,
    sent: VecDeque<Duration>,
}

impl Pacer {
    pub fn new(clock: Arc<dyn Clock>, per_minute: usize) -> Self {
        Self {
            clock,
            per_minute: per_minute.max(1),
            sent: VecDeque::new(),
        }
    }

    pub async fn wait_turn(&mut self) {
        loop {
            let now = self.clock.now();
            while self
                .sent
                .front()
                .is_some_and(|oldest| now.saturating_sub(*oldest) >= WINDOW)
            {
                self.sent.pop_front();
            }
            let Some(oldest) = self.sent.front().copied() else {
                break;
            };
            if self.sent.len() < self.per_minute {
                break;
            }
            self.clock
                .sleep(WINDOW.saturating_sub(now.saturating_sub(oldest)))
                .await;
        }
        self.sent.push_back(self.clock.now());
    }
}
