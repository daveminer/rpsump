use crate::hydro::gpio::Gpio;
use crate::{config::HeaterConfig, hydro::control::Control};
use anyhow::Error;
use tracing::error;

#[derive(Clone)]
pub struct Heater {
    pub control: Control,
}

impl Heater {
    pub fn new(config: &HeaterConfig, gpio: &Box<dyn Gpio>) -> Result<Self, Error> {
        let control = Control::new("Pool Heater".into(), config.control_pin, gpio)?;
        Ok(Self { control })
    }

    pub async fn on(&mut self) -> Result<(), Error> {
        let pin = self.control.pin.clone();

        let _ = tokio::task::spawn_blocking(move || {
            let mut lock = match pin.lock() {
                Ok(lock) => lock,
                Err(e) => {
                    tracing::error!(
                        target = module_path!(),
                        error = e.to_string(),
                        "Could not lock heater pin for on"
                    );
                    return;
                }
            };
            lock.on();
        });

        Ok(())
    }

    pub async fn off(&mut self) -> Result<(), Error> {
        let pin = self.control.pin.clone();

        let _ = tokio::task::spawn_blocking(move || {
            let mut lock = match pin.lock() {
                Ok(lock) => lock,
                Err(e) => {
                    tracing::error!(
                        target = module_path!(),
                        error = e.to_string(),
                        "Could not lock heater pin for off"
                    );
                    return;
                }
            };
            lock.off();
        });

        Ok(())
    }

    pub async fn is_on(&self) -> bool {
        match self.control.lock().await {
            Ok(guard) => guard.is_on(),
            Err(e) => {
                error!("Error locking heater pin for is_on: {}", e);
                false
            }
        }
    }

    pub async fn is_off(&self) -> bool {
        match self.control.lock().await {
            Ok(guard) => guard.is_off(),
            Err(e) => {
                error!("Error locking heater pin for is_off: {}", e);
                false
            }
        }
    }
}
