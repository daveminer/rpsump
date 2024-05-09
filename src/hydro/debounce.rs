use anyhow::Error;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::{Mutex, Notify};
use tokio::time::{Duration, Instant};

use crate::hydro::{
    gpio::Level,
    signal::{Message, Signal},
};

#[derive(Clone, Debug)]
pub struct Debouncer {
    deadline: Instant,
    delay: Duration,
    level: Arc<Mutex<Level>>,
    message: Message,
    tx: Sender<Signal>,
    reset_signal: Arc<Notify>,
}

/// Tracks the original level of a sensor pin and will reset deadline along with the new state
/// when a change to the sensor pin state occurs before the deadline elapses. Otherwise, water
/// turbulence may trigger multiple events when only one is desired.
impl Debouncer {
    pub fn new(level: Level, delay: Duration, message: Message, tx: Sender<Signal>) -> Self {
        let debouncer = Self {
            deadline: Instant::now() + delay,
            delay,
            level: Arc::from(Mutex::new(level)),
            message,
            tx,
            reset_signal: Arc::new(Notify::new()),
        };
        debouncer.start();
        debouncer
    }

    pub async fn reset_deadline(&mut self, level: Level) -> Result<(), Error> {
        let mut lock = self.level.lock().await;
        *lock = level;
        drop(lock);

        self.deadline = Instant::now() + self.delay;
        self.reset_signal.notify_one();

        Ok(())
    }

    fn start(&self) {
        let reset_signal = Arc::clone(&self.reset_signal);
        let tx = self.tx.clone();
        let message = self.message.clone();
        let sleep_duration = self.deadline.saturating_duration_since(Instant::now());
        let level = Arc::clone(&self.level);

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = tokio::time::sleep(sleep_duration) => {
                        let lock = level.lock().await;
                        let signal = Signal {level: *lock, message: message.clone()};

                        tx.send(signal).await.unwrap();
                        break;
                    }
                    _ = reset_signal.notified() => {
                        // The deadline was reset, restart the sleep
                        continue;
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hydro::signal::Signal;
    use std::time::Duration;
    use tokio::sync::mpsc::Receiver;

    #[tokio::test]
    async fn test_reset_deadline() {
        let (tx, mut rx): (Sender<Signal>, Receiver<Signal>) = tokio::sync::mpsc::channel(32);

        let mut debouncer = Debouncer::new(
            Level::High,
            Duration::from_secs(1),
            Message::SumpEmpty,
            tx.clone(),
        );
        debouncer.reset_deadline(Level::Low).await.unwrap();

        // Wait for the message to be sent
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Check that the message was sent
        assert_eq!(
            rx.recv().await,
            Some(Signal {
                level: Level::Low,
                message: Message::SumpEmpty
            })
        );
    }

    #[tokio::test]
    async fn test_message_sent_after_delay() {
        let (tx, mut rx): (Sender<Signal>, Receiver<Signal>) = tokio::sync::mpsc::channel(32);

        let _debouncer = Debouncer::new(
            Level::High,
            Duration::from_secs(1),
            Message::SumpFull,
            tx.clone(),
        );

        // Wait for the message to be sent
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Check that the message was sent
        assert_eq!(
            rx.recv().await,
            Some(Signal {
                level: Level::High,
                message: Message::SumpFull
            })
        );
    }
}
