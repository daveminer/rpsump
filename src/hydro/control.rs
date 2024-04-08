use crate::hydro::gpio::{Gpio, OutputPin};
use crate::util::spawn_blocking_with_tracing;
use anyhow::Error;
use async_trait::async_trait;
use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};

use tracing::error;

pub type PinLock<'a> = MutexGuard<'a, Box<dyn OutputPin>>;

/// Represents a GPIO output for controlling stateful equipment. Water pump, etc.
pub type SharedOutputPin = Arc<Mutex<Box<dyn OutputPin>>>;

#[derive(Clone)]
pub struct Control {
    pub label: String,
    pub pin: SharedOutputPin,
}

impl Control {
    /// Creates a new output on a GPIO pin.
    pub fn new<G>(label: String, pin: u8, gpio: &G) -> Result<Self, Error>
    where
        G: Gpio,
    {
        let pin = gpio.get(pin)?;
        let pin_io = pin.into_output_low();

        Ok(Self {
            label,
            pin: Arc::from(Mutex::new(pin_io)),
        })
    }

    pub async fn lock(&self) -> Result<PinLock, Error> {
        self.pin.lock().map_err(|e| Error::msg(e.to_string()))
    }
}

impl fmt::Debug for Control {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Control")
            .field("label", &self.label)
            .finish()
    }
}

#[async_trait]
pub trait Output {
    async fn on(&mut self) -> Result<(), Error>;
    async fn off(&mut self) -> Result<(), Error>;
    fn is_on(&self) -> bool;
    fn is_off(&self) -> bool;
}

#[async_trait]
impl Output for Control {
    async fn on(&mut self) -> Result<(), Error> {
        let pin = self.pin.clone();

        let _ = spawn_blocking_with_tracing(move || match pin.lock() {
            Ok(mut guard) => guard.on(),
            Err(e) => error!("Error locking pin for on: {}", e),
        })
        .await?;

        Ok(())
    }

    async fn off(&mut self) -> Result<(), Error> {
        let pin = self.pin.clone();

        let _ = spawn_blocking_with_tracing(move || match pin.lock() {
            Ok(mut guard) => guard.off(),
            Err(e) => error!("Error locking pin for off: {}", e),
        })
        .await?;

        Ok(())
    }

    fn is_on(&self) -> bool {
        match self.pin.lock() {
            Ok(guard) => guard.is_on(),
            Err(e) => {
                error!("Error locking pin for is_on: {}", e);
                false
            }
        }
    }

    fn is_off(&self) -> bool {
        match self.pin.lock() {
            Ok(guard) => guard.is_off(),
            Err(e) => {
                error!("Error locking pin for is_off: {}", e);
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Control;

    #[cfg(test)]
    mod tests {
        use super::Control;
        use crate::hydro::control::Output;
        use crate::test_fixtures::gpio::mock_gpio_get;

        #[tokio::test]
        async fn test_control_new() {
            let mock_gpio = mock_gpio_get(vec![1]);

            let control = Control::new("test control".to_string(), 1, &mock_gpio);

            assert!(control.is_ok());
        }

        #[tokio::test]
        async fn test_control_on_off() {
            let mock_gpio = mock_gpio_get(vec![1]);

            let mut control = Control::new("test control".to_string(), 1, &mock_gpio).unwrap();

            assert!(control.on().await.is_ok());
            assert_eq!(control.is_on(), true);

            assert!(control.off().await.is_ok());
            assert_eq!(control.is_off(), true);
        }
    }
}
