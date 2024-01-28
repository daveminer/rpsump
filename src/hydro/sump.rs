use anyhow::Error;
use std::process::Command;
use tokio::{runtime::Handle, sync::mpsc::Sender};

use crate::{
    config::SumpConfig,
    hydro::{
        gpio::{Gpio, Trigger},
        sensor::Sensor,
        Control,
    },
};

#[derive(Clone)]
pub struct Sump {
    pub high_sensor: Sensor,
    pub low_sensor: Sensor,
    pub pump: Control,
}

impl Sump {
    pub fn new<G>(
        config: &SumpConfig,
        tx: &Sender<Command>,
        handle: &Handle,
        gpio: &G,
    ) -> Result<Self, Error>
    where
        G: Gpio,
    {
        let pump = Control::new("sump pump".into(), config.pump_control_pin, gpio)?;

        let high_sensor = Sensor::new(
            "Sump Full".to_string(),
            config.high_sensor_pin,
            gpio,
            Trigger::Both,
            handle,
            tx,
            0,
        )?;

        let low_sensor = Sensor::new(
            "Sump Empty".to_string(),
            config.low_sensor_pin,
            gpio,
            Trigger::Both,
            handle,
            tx,
            // TODO: verify
            1000,
        )?;

        Ok(Self {
            high_sensor,
            low_sensor,
            pump,
        })
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::{
//         config::SumpConfig,
//         hydro::gpio::{stub::pin, Level, MockGpio},
//     };

//     #[test]
//     fn test_new() {
//         let config = SumpConfig {
//             enabled: true,
//             high_sensor_pin: 1,
//             low_sensor_pin: 2,
//             pump_control_pin: 3,
//             pump_shutoff_delay: 4,
//         };

//         let mut mock_gpio = MockGpio::new();
//         mock_gpio.expect_get().times(3).returning(|_| {
//             Ok(Box::new(pin::PinStub {
//                 index: 0,
//                 level: Level::Low,
//             }))
//         });

//         //TODO: finish
//         //let _sump: Sump = Sump::new(&config, &mock_gpio, ).unwrap();
//     }
// }
