use anyhow::Error;
use tokio::{runtime::Handle, sync::mpsc::Sender};

use crate::{
    config::SumpConfig,
    hydro::{
        gpio::{Gpio, Trigger},
        sensor::Sensor,
        signal::Message,
        Control,
    },
};

use super::signal::Signal;

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
    pub fn new(
        config: &SumpConfig,
        tx: &Sender<Signal>,
        handle: Handle,
        gpio: &dyn Gpio,
    ) -> Result<Self, Error> {
        let pump = Control::new("Sump Pump".into(), config.pump_control_pin, gpio)?;

        let high_sensor = Sensor::new(
            Message::SumpFull,
            config.high_sensor_pin,
            gpio,
            Trigger::RisingEdge,
            tx,
            handle.clone(),
        )?;

        let low_sensor = Sensor::new(
            Message::SumpEmpty,
            config.low_sensor_pin,
            gpio,
            Trigger::FallingEdge,
            tx,
            handle.clone(),
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
    use tokio::runtime::Runtime;

    use crate::{
        hydro::gpio::MockGpio,
        test_fixtures::{gpio::mock_sump_pump, settings::SETTINGS},
    };

    use super::Sump;

    #[test]
    fn test_new() {
        let mpsc = tokio::sync::mpsc::channel(32);
        let rt = Runtime::new().unwrap();
        let handle = rt.handle();

        let mock_gpio = mock_sump_pump(MockGpio::new(), false, false, false);
        let _sump: Sump =
            Sump::new(&SETTINGS.hydro.sump, &mpsc.0, handle.clone(), &mock_gpio).unwrap();
    }
}
