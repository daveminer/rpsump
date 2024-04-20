use crate::hydro::gpio::{Gpio, OutputPin};
use crate::util::spawn_blocking_with_tracing;
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
    pub pin: SharedOutputPin,
}

impl Control {
    /// Creates a new output on a GPIO pin.
    pub fn new(label: String, pin: u8, gpio: &dyn Gpio) -> Result<Self, Error> {
        let pin = gpio.get(pin)?;
        let pin_io = pin.into_output_low();

        Ok(Self {
            label,
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
            .finish()
    }
}

impl PartialEq for Control {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label
    }
}

#[async_trait]
pub trait Output {
    async fn on(&mut self) -> Result<(), Error>;
    async fn off(&mut self) -> Result<(), Error>;
    async fn is_on(&self) -> bool;
    async fn is_off(&self) -> bool;
}

#[async_trait]
impl Output for Control {
    async fn on(&mut self) -> Result<(), Error> {
        let self = self.clone();
        spawn_blocking_with_tracing(|| async move {
            let pin = self.pin.clone();
            let mut lock = pin.lock().await;
            lock.on();
        })
        .await?
        .await;

        Ok(())
    }

    async fn off(&mut self) -> Result<(), Error> {
        let self = self.clone();
        spawn_blocking_with_tracing(|| async move {
            let pin = self.pin.clone();
            let mut lock = pin.lock().await;
            lock.off();
        })
        .await?
        .await;

        Ok(())
    }

    async fn is_on(&self) -> bool {
        let lock = self.pin.lock().await;
        lock.is_on()
    }

    async fn is_off(&self) -> bool {
        let lock = self.pin.lock().await;
        lock.is_off()
    }
}

#[cfg(test)]
mod tests {
    use super::Control;
    use crate::test_fixtures::gpio::mock_control_gpio;

    #[tokio::test]
    async fn test_control_new() {
        let control = Control::new("test control".to_string(), 1, &mock_control_gpio());

        assert!(control.is_ok());
    }
}
