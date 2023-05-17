/*  */
use std::sync::{Arc, Mutex};
use tokio::time::{Duration, Instant};

use crate::database::DbPool;
use crate::sump::{SharedOutputPin, SharedPinState};
use rppal::gpio::{Level, OutputPin};

#[derive(Clone, Debug)]
pub struct SensorDebouncer {
    db: DbPool,
    func: fn(Level, Arc<Mutex<OutputPin>>, SharedPinState, u64, DbPool) -> (),
    inner: Arc<Mutex<DebouncerInner>>,
    prev_reading: Level,
    pump_control_pin: SharedOutputPin,
    sensor_state: SharedPinState,
    delay: u64,
}

#[derive(Clone, Debug)]
struct DebouncerInner {
    deadline: Instant,
    duration: Duration,
}

impl SensorDebouncer {
    pub fn new(
        duration: Duration,
        level: Level,
        pump_control_pin: SharedOutputPin,
        sensor_state: SharedPinState,
        func: fn(Level, SharedOutputPin, SharedPinState, u64, DbPool) -> (),
        delay: u64,
        db: DbPool,
    ) -> Self {
        Self {
            db,
            func,
            inner: Arc::new(Mutex::new(DebouncerInner {
                deadline: Instant::now() + duration,
                duration,
            })),
            prev_reading: level,
            pump_control_pin,
            sensor_state,
            delay,
        }
    }

    // Reset the deadline, increasing the duration of any calls to `sleep`.
    pub fn reset_deadline(&mut self, new_reading: Level) {
        if self.prev_reading != new_reading {
            self.prev_reading = new_reading;
            let mut lock = self.inner.lock().unwrap();
            lock.deadline = Instant::now() + lock.duration;
        }
    }

    // Sleep until the deadline elapses.
    pub async fn sleep(&self) {
        // This uses a loop in case the deadline has been reset since the
        // sleep started, in which case the code will sleep again.
        loop {
            let deadline = self.get_deadline();
            if deadline <= Instant::now() {
                // The deadline has already elapsed. Just return.
                (self.func)(
                    self.prev_reading,
                    self.pump_control_pin.clone(),
                    self.sensor_state.clone(),
                    self.delay.clone(),
                    self.db.clone(),
                );
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
