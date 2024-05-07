use anyhow::{anyhow, Error};
use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc::Sender;

use crate::hydro::{
    debounce::Debouncer,
    gpio::{Gpio, InputPin, Trigger},
    signal::Message,
    Level,
};

/// Sensors will trigger async callbacks (which create a thread) on these
pub type SharedInputPin = Arc<Mutex<Box<dyn InputPin>>>;
/// Threads spawned from sensor state changes will share one of these per sensor
pub type SharedSensorDebouncer = Arc<Mutex<Option<Debouncer>>>;

#[derive(Clone, Debug)]
pub struct Sensor {
    pub level: Level,
    pub pin: SharedInputPin,
    pub debounce: SharedSensorDebouncer,
}

pub trait Input {
    fn is_high(&self) -> bool;
    fn is_low(&self) -> bool;
    fn level(&self) -> Level;
}

/// Represents a GPIO output for controlling stateful equipment. Water pump, etc.
/// Can be read synchronously or listened to for events.
///
/// # Arguments
///
/// * `name` - The name of the sensor for labelling
/// * `pin_number` - The GPIO pin number to listen to
/// * `gpio` - The GPIO implementation to use
/// * `trigger` - The trigger to listen for; rising, falling, or both
/// * `handle` - handler function to run when the trigger is detected
/// * `tx` - The channel to send commands to
/// * `delay` - The debounce delay in milliseconds, if any. Used to
///    leave the pump on momentarily after a low water level is detected.
impl Sensor {
    pub fn new(
        message: Message,
        pin_number: u8,
        gpio: &dyn Gpio,
        trigger: Trigger,
        tx: &Sender<Message>,
    ) -> Result<Self, Error> {
        let mut pin_io = gpio
            .get(pin_number)
            .map_err(|e| anyhow!(e))?
            .into_input_pullup();

        let debounce = Arc::from(Mutex::new(None));

        pin_io
            .set_async_interrupt(message, trigger, tx, debounce.clone())
            .map_err(|e| anyhow!(e.to_string()))?;

        Ok(Self {
            level: pin_io.read(),
            pin: Arc::from(Mutex::new(pin_io)),
            debounce,
        })
    }
}

impl Input for Sensor {
    fn is_high(&self) -> bool {
        self.level == Level::High
    }

    fn is_low(&self) -> bool {
        self.level == Level::Low
    }

    fn level(&self) -> Level {
        self.level
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        hydro::{gpio::Trigger, signal::Message},
        test_fixtures::gpio::mock_sensor_gpio,
    };

    use super::Sensor;

    #[test]
    fn test_new() {
        let (tx, _) = tokio::sync::mpsc::channel(32);

        let _sensor: Sensor = Sensor::new(
            Message::IrrigatorEmpty,
            1,
            &mock_sensor_gpio(),
            Trigger::Both,
            &tx,
        )
        .unwrap();
    }
}
