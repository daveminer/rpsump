use crate::hydro::gpio::Gpio;
use crate::{config::HeaterConfig, hydro::control::Control};
use anyhow::Error;

#[derive(Clone)]
pub struct Heater {
    pub control: Control,
}

impl Heater {
    pub fn new(config: &HeaterConfig, gpio: &dyn Gpio) -> Result<Self, Error> {
        let control = Control::new("Pool Heater".into(), config.control_pin, gpio)?;
        Ok(Self { control })
    }

    pub async fn on(&mut self) {
        let pin = self.control.pin.clone();
        let mut lock = pin.lock().await;

        lock.on();
    }

    pub async fn off(&mut self) {
        let pin = self.control.pin.clone();
        let mut lock = pin.lock().await;

        lock.off();
    }

    pub async fn is_on(&self) -> bool {
        let lock = self.control.lock().await;

        lock.is_on()
    }

    pub async fn is_off(&self) -> bool {
        let lock = self.control.lock().await;

        lock.is_off()
    }
}
