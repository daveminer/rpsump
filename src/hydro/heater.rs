use crate::hydro::gpio::Gpio;
use crate::{config::HeaterConfig, hydro::control::Control};
use anyhow::Error;

#[derive(Clone)]
pub struct Heater {
    control: Control,
}

impl Heater {
    pub fn new<G>(config: &HeaterConfig, gpio: &G) -> Result<Self, Error>
    where
        G: Gpio,
    {
        let control = Control::new("poop pump".into(), config.control_pin, gpio)?;
        Ok(Self { control })
    }
    pub async fn on(self) -> Result<(), Error> {
        let mut lock = self.control.lock().await;
        lock.set_high();

        Ok(())
    }

    pub async fn off(self) -> Result<(), Error> {
        let mut lock = self.control.lock().await;
        lock.set_low();

        Ok(())
    }
}
