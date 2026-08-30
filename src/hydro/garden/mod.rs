pub mod schedule;

use anyhow::Error;
use std::sync::Arc;
use tokio::sync::Notify;

use crate::config::GardenConfig;
use crate::hydro::control::Control;
use crate::hydro::gpio::Gpio;

#[derive(Clone, Debug)]
pub struct Garden {
    pub solenoid: Control,
    pub max_seconds_runtime: u32,
    /// Cuts the scheduler's sleep short so a run queued over HTTP starts now
    /// instead of at the next tick.
    run_signal: Arc<Notify>,
}

impl Garden {
    pub fn new(config: &GardenConfig, gpio: &dyn Gpio) -> Result<Self, Error> {
        let solenoid = Control::new("Garden Solenoid".into(), config.solenoid_pin, gpio)?;
        Ok(Self {
            solenoid,
            max_seconds_runtime: config.max_seconds_runtime,
            run_signal: Arc::new(Notify::new()),
        })
    }

    /// Wakes the scheduler immediately. A signal sent while the scheduler is
    /// busy is remembered, so the next sleep returns at once.
    pub fn wake_scheduler(&self) {
        self.run_signal.notify_one();
    }

    /// Resolves on the next `wake_scheduler`.
    pub async fn woken(&self) {
        self.run_signal.notified().await;
    }

    pub async fn is_on(&self) -> bool {
        let lock = self.solenoid.lock().await;
        lock.is_on()
    }
}
