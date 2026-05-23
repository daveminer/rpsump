pub mod schedule;

use anyhow::Error;

use crate::config::GardenConfig;
use crate::hydro::control::Control;
use crate::hydro::gpio::Gpio;

#[derive(Clone, Debug)]
pub struct Garden {
    pub solenoid: Control,
    pub max_seconds_runtime: u32,
}

impl Garden {
    pub fn new(config: &GardenConfig, gpio: &dyn Gpio) -> Result<Self, Error> {
        let solenoid = Control::new("Garden Solenoid".into(), config.solenoid_pin, gpio)?;
        Ok(Self {
            solenoid,
            max_seconds_runtime: config.max_seconds_runtime,
        })
    }

    pub async fn is_on(&self) -> bool {
        let lock = self.solenoid.lock().await;
        lock.is_on()
    }
}
