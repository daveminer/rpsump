use crate::{
    config::IrrigationConfig,
    database::DbPool,
    hydro::{
        gpio::{Gpio, Trigger},
        Control, Level, Sensor,
    },
};
use anyhow::Error;

#[derive(Clone)]
pub struct Irrigator {
    pub low_sensor: Sensor,
    pub pump: Control,
    pub valve1: Control,
    pub valve2: Control,
    pub valve3: Control,
    pub valve4: Control,
}

impl Irrigator {
    pub fn new<C, G>(
        db: DbPool,
        config: &IrrigationConfig,
        gpio: &G,
        low_sensor_handler: C,
    ) -> Result<Self, Error>
    where
        C: FnMut(Level) + Send + 'static,
        G: Gpio,
    {
        let low_sensor = Sensor::new(
            config.low_sensor_pin,
            gpio,
            Some(low_sensor_handler),
            Some(Trigger::Both),
        )?;
        let pump = Control::new("irrigation pump".into(), config.pump_control_pin, gpio)?;
        let valve1 = Control::new(
            "irrigation valve 1".into(),
            config.valve_1_control_pin,
            gpio,
        )?;
        let valve2 = Control::new(
            "irrigation valve 2".into(),
            config.valve_2_control_pin,
            gpio,
        )?;
        let valve3 = Control::new(
            "irrigation valve 3".into(),
            config.valve_3_control_pin,
            gpio,
        )?;
        let valve4 = Control::new(
            "irrigation valve 4".into(),
            config.valve_4_control_pin,
            gpio,
        )?;

        Ok(Self {
            low_sensor,
            pump,
            valve1,
            valve2,
            valve3,
            valve4,
        })
    }
}
