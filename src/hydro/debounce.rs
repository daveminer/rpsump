use std::sync::{Arc, Mutex};
use tokio::time::{Duration, Instant};

use rppal::gpio::Level;

#[derive(Clone, Debug)]
pub struct Debouncer {
    inner: Arc<Mutex<DebouncerInner>>,
    prev_reading: Level,
}

#[derive(Clone, Debug)]
struct DebouncerInner {
    deadline: Instant,
    duration: Duration,
}

/// Tracks the original level of a sensor pin and will reset deadline along with the new state
/// when a change to the sensor pin state occurs before the deadline elapses. Otherwise, water
/// turbulence may trigger multiple events where only one is desired.
impl Debouncer {
    pub fn new(duration: Duration, level: Level) -> Self {
        Self {
            inner: Arc::new(Mutex::new(DebouncerInner {
                deadline: Instant::now() + duration,
                duration,
            })),
            prev_reading: level,
        }
    }

    /// Reset the deadline, increasing the duration of any calls to `sleep` and updating the reading.
    #[tracing::instrument]
    pub fn reset_deadline(&mut self, new_reading: Level) {
        if self.prev_reading != new_reading {
            self.prev_reading = new_reading;
            let mut lock = self.inner.lock().unwrap();
            lock.deadline = Instant::now() + lock.duration;
        }
    }

    /// Sleeps until the deadline elapses.
    #[tracing::instrument]
    pub async fn sleep(&self) {
        // This uses a loop in case the deadline has been reset since the
        // sleep started, in which case the code will sleep again.
        loop {
            let deadline = self.get_deadline();
            if deadline <= Instant::now() {
                // The deadline has elapsed; just return.
                return;
            }
            tokio::time::sleep_until(deadline).await;
        }
    }

    #[tracing::instrument]
    pub fn get_deadline(&self) -> Instant {
        let lock = self.inner.lock().unwrap();
        lock.deadline
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::Instant;

    #[tokio::test]
    async fn test_reset_deadline() {
        let mut debouncer = Debouncer::new(Duration::from_secs(1), Level::High);
        debouncer.reset_deadline(Level::Low);
        assert_eq!(debouncer.prev_reading, Level::Low);
    }

    #[tokio::test]
    async fn test_sleep() {
        let debouncer = Debouncer::new(Duration::from_secs(1), Level::High);
        let start = Instant::now();
        debouncer.sleep().await;
        assert!(Instant::now() - start >= Duration::from_secs(1));
    }

    #[tokio::test]
    async fn test_get_deadline() {
        let debouncer = Debouncer::new(Duration::from_secs(1), Level::High);
        let deadline = debouncer.get_deadline();
        assert!(deadline <= Instant::now() + Duration::from_secs(1));
    }
}
