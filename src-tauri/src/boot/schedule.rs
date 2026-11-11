use std::future::Future;
use std::time::Duration;

pub const BACKUP_RECHECK_INTERVAL: Duration = Duration::from_secs(3600);

pub async fn run_every<F, Fut>(period: Duration, mut tick: F)
where
    F: FnMut() -> Fut,
    Fut: Future<Output = ()>,
{
    let mut timer = tokio::time::interval(period);
    timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    loop {
        timer.tick().await;
        tick().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test(start_paused = true)]
    async fn the_first_round_is_immediate_then_one_per_period() {
        let rounds = Arc::new(AtomicU32::new(0));
        let counter = rounds.clone();
        let handle = tokio::spawn(run_every(BACKUP_RECHECK_INTERVAL, move || {
            let counter = counter.clone();
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
            }
        }));
        for _ in 0..5 {
            tokio::task::yield_now().await;
        }
        assert_eq!(rounds.load(Ordering::SeqCst), 1);
        for expected in 2..=4 {
            tokio::time::advance(BACKUP_RECHECK_INTERVAL).await;
            for _ in 0..5 {
                tokio::task::yield_now().await;
            }
            assert_eq!(rounds.load(Ordering::SeqCst), expected);
        }
        handle.abort();
    }
}
