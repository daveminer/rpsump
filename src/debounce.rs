use std::sync::{Arc, Mutex};
use tokio::time::{Duration, Instant};

use rppal::gpio::Level;

#[derive(Clone, Debug)]
pub struct SensorDebouncer {
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
impl SensorDebouncer {
    pub fn new(duration: Duration, level: Level) -> Self {
        println!("NEW DEBOUNCE");
        Self {
            inner: Arc::new(Mutex::new(DebouncerInner {
                deadline: Instant::now() + duration,
                duration,
            })),
            prev_reading: level,
        }
    }

    /// Reset the deadline, increasing the duration of any calls to `sleep` and updating the reading.
    pub fn reset_deadline(&mut self, new_reading: Level) {
        println!("CHECKING RESET: {:?} {:?}", self.prev_reading, new_reading);
        if self.prev_reading != new_reading {
            println!("RESETTING DEADLINE");
            self.prev_reading = new_reading;
            let mut lock = self.inner.lock().unwrap();
            lock.deadline = Instant::now() + lock.duration;
        }
    }

    /// Sleeps until the deadline elapses.
    pub async fn sleep(&self) {
        // This uses a loop in case the deadline has been reset since the
        // sleep started, in which case the code will sleep again.
        println!("SLEEPING");
        loop {
            let deadline = self.get_deadline();
            if deadline <= Instant::now() {
                // The deadline has already elapsed. Just return.
                println!("DEADLINE ELAPSED");
                return;
            }
            tokio::time::sleep_until(deadline).await;
        }
    }

    fn get_deadline(&self) -> Instant {
        let lock = self.inner.lock().unwrap();
        lock.deadline
    }
}
