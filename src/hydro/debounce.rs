use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::Sender;
use tokio::time::{Duration, Instant};

use crate::hydro::{gpio::Level, signal::Message};

#[derive(Clone, Debug)]
pub struct Debouncer {
    inner: Arc<Mutex<DebouncerInner>>,
    message: Message,
    prev_reading: Level,
    tx: Sender<Message>,
}

#[derive(Clone, Debug)]
struct DebouncerInner {
    deadline: Instant,
    duration: Duration,
}

/// Tracks the original level of a sensor pin and will reset deadline along with the new state
/// when a change to the sensor pin state occurs before the deadline elapses. Otherwise, water
/// turbulence may trigger multiple events when only one is desired.
impl Debouncer {
    pub fn new(level: Level, duration: Duration, message: Message, tx: Sender<Message>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(DebouncerInner {
                deadline: Instant::now() + duration,
                duration,
            })),
            message,
            prev_reading: level,
            tx,
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

    /// Sleeps until the deadline elapses, then sends the trigger event.
    #[tracing::instrument]
    pub async fn sleep(&self) {
        // This uses a loop in case the deadline has been reset since the
        // sleep started, in which case the code will sleep again.
        loop {
            let deadline = self.get_deadline();
            if deadline <= Instant::now() {
                // The deadline has elapsed; send the trigger event.
                self.tx.send(self.message.clone()).await.unwrap();

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
    use tokio::{sync::mpsc::Receiver, time::Instant};

    #[tokio::test]
    async fn test_reset_deadline() {
        let mpsc: (Sender<Message>, Receiver<Message>) = tokio::sync::mpsc::channel(32);

        let mut debouncer = Debouncer::new(
            Level::High,
            Duration::from_secs(1),
            Message::SumpEmpty,
            mpsc.0,
        );
        debouncer.reset_deadline(Level::Low);
        assert_eq!(debouncer.prev_reading, Level::Low);
    }

    #[tokio::test]
    async fn test_sleep() {
        let mpsc: (Sender<Message>, Receiver<Message>) = tokio::sync::mpsc::channel(32);

        let debouncer = Debouncer::new(
            Level::High,
            Duration::from_secs(1),
            Message::SumpFull,
            mpsc.0,
        );
        let start = Instant::now();
        debouncer.sleep().await;
        assert!(Instant::now() - start >= Duration::from_secs(1));
    }

    #[tokio::test]
    async fn test_get_deadline() {
        let mpsc: (Sender<Message>, Receiver<Message>) = tokio::sync::mpsc::channel(32);

        let debouncer = Debouncer::new(
            Level::High,
            Duration::from_secs(1),
            Message::IrrigatorEmpty,
            mpsc.0,
        );
        let deadline = debouncer.get_deadline();
        assert!(deadline <= Instant::now() + Duration::from_secs(1));
    }
}
