use anyhow::Error;
use tokio::sync::mpsc::Sender;

use crate::{
    config::SumpConfig,
    hydro::{
        gpio::{Gpio, Trigger},
        sensor::Sensor,
        signal::Message,
        Control,
    },
};

#[derive(Clone)]
pub struct Sump {
    pub high_sensor: Sensor,
    pub low_sensor: Sensor,
    pub pump: Control,
}

/// Sumps controls the GPIOs for devices (water level sensors and a water pump)
/// that measure the water in a reservoir and pump it out when it gets full.
///
/// # Arguments
///
/// * `config`  - The configuration for the sump
/// * `tx`      - The channel used to report triggers to the main channel
/// * `handle`  - The callback for trigger events. Uses the `tx` channel.
/// * `gpio`    - The GPIO interface to use for the sump
///
impl Sump {
    /// Create a new instance of Sump with the provided configuration, GPIO,
    /// trigger callback handle and tx channel to report triggers upon
    pub fn new<G>(config: &SumpConfig, tx: &Sender<Message>, gpio: &G) -> Result<Self, Error>
    where
        G: Gpio,
    {
        let pump = Control::new("sump pump".into(), config.pump_control_pin, gpio)?;

        let high_sensor = Sensor::new(
            Message::SumpFull,
            config.high_sensor_pin,
            gpio,
            Trigger::Both,
            tx,
        )?;

        let low_sensor = Sensor::new(
            Message::SumpEmpty,
            config.low_sensor_pin,
            gpio,
            Trigger::Both,
            tx,
        )?;

        Ok(Self {
            high_sensor,
            low_sensor,
            pump,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        config::SumpConfig,
        hydro::gpio::{stub::pin, Level, MockGpio},
    };

    use super::Sump;

    #[test]
    fn test_new() {
        let config = SumpConfig {
            enabled: true,
            high_sensor_pin: 1,
            low_sensor_pin: 2,
            pump_control_pin: 3,
            pump_shutoff_delay: 4,
        };

        let mpsc = tokio::sync::mpsc::channel(32);

        let mut mock_gpio = MockGpio::new();
        mock_gpio.expect_get().times(3).returning(|_| {
            Ok(Box::new(pin::PinStub {
                index: 0,
                level: Level::Low,
            }))
        });

        let _sump: Sump = Sump::new(&config, &mpsc.0, &mock_gpio).unwrap();
    }
}
