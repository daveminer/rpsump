use crate::hydro::gpio::{Gpio, Level, OutputPin};
use anyhow::Error;
use async_trait::async_trait;
use std::fmt;
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};

pub type PinLock<'a> = MutexGuard<'a, Box<dyn OutputPin>>;

/// Represents a GPIO output for controlling stateful equipment. Water pump, etc.
pub type SharedOutputPin = Arc<Mutex<Box<dyn OutputPin>>>;

#[derive(Clone)]
pub struct Control {
    pub label: String,
    pub level: Level,
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
            level: Level::Low,
            pin: Arc::from(Mutex::new(pin_io)),
        })
    }

    pub async fn lock(&self) -> PinLock {
        self.pin.lock().await
    }
}

impl fmt::Debug for Control {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Control")
            .field("label", &self.label)
            .field("level", &self.level)
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
        self.level = Level::High;
        let mut pin = self.lock().await;

        pin.on();

        Ok(())
    }

    async fn off(&mut self) -> Result<(), Error> {
        self.level = Level::Low;
        let mut pin = self.lock().await;

        pin.off();

        Ok(())
    }

    fn is_on(&self) -> bool {
        self.level == Level::High
    }

    fn is_off(&self) -> bool {
        self.level == Level::Low
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
