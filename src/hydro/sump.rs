use anyhow::Error;

use crate::{
    config::SumpConfig,
    hydro::{
        gpio::{Gpio, Level, Trigger},
        Control, Sensor,
    },
};

#[derive(Clone)]
pub struct Sump {
    pub high_sensor: Sensor,
    pub low_sensor: Sensor,
    pub pump: Control,
}

impl Sump {
    pub fn new<C, G>(
        config: &SumpConfig,
        gpio: &G,
        high_sensor_handler: C,
        low_sensor_handler: C,
    ) -> Result<Self, Error>
    where
        C: FnMut(Level) -> () + Send + 'static,
        G: Gpio,
    {
        let high_sensor = Sensor::new(
            config.high_sensor_pin,
            gpio,
            Some(high_sensor_handler),
            Some(Trigger::Both),
        )?;
        let low_sensor = Sensor::new(
            config.low_sensor_pin,
            gpio,
            Some(low_sensor_handler),
            Some(Trigger::Both),
        )?;
        let pump = Control::new("sump pump".into(), config.pump_control_pin, gpio)?;

        Ok(Self {
            high_sensor,
            low_sensor,
            pump,
        })
    }

    // fn status(&self) -> String {
    //     if self.pump.is_on() {
    //         "pumping".into()
    //     } else if self.low_sensor.is_low() {
    //         "empty".into()
    //     } else {
    //         "filling".into()
    //     }
    // }
}
