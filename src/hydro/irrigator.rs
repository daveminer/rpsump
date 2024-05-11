use anyhow::Error;
use tokio::{runtime::Handle, sync::mpsc::Sender};

use crate::{
    config::IrrigationConfig,
    hydro::{
        gpio::{Gpio, Trigger},
        sensor::Sensor,
        signal::Message,
        Control,
    },
};

use super::signal::Signal;

#[derive(Clone, Debug)]
pub struct Irrigator {
    pub low_sensor: Sensor,
    pub pump: Control,
    pub valve1: Control,
    pub valve2: Control,
    pub valve3: Control,
    pub valve4: Control,
}

impl Irrigator {
    pub fn new(
        config: &IrrigationConfig,
        tx: &Sender<Signal>,
        handle: Handle,
        gpio: &dyn Gpio,
    ) -> Result<Self, Error> {
        let pump = Control::new("Irrigation Pump".to_string(), config.pump_control_pin, gpio)?;

        let low_sensor = Sensor::new(
            Message::IrrigatorEmpty,
            config.low_sensor_pin,
            gpio,
            Trigger::Both,
            tx,
            handle,
        )?;

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

#[cfg(test)]
mod tests {
    use tokio::runtime::Runtime;

    use crate::{
        hydro::gpio::{Level, MockGpio},
        test_fixtures::{gpio::mock_irrigation_pump, settings::SETTINGS},
    };

    use super::Irrigator;

    #[test]
    fn test_new() {
        let mpsc = tokio::sync::mpsc::channel(32);
        let rt = Runtime::new().unwrap();
        let handle = rt.handle().clone();

        let mut mock_gpio = MockGpio::new();
        mock_gpio = mock_irrigation_pump(mock_gpio, false, Level::High, false, None);

        let _irrigator: Irrigator = Irrigator::new(
            &SETTINGS.hydro.irrigation,
            &mpsc.0,
            handle.clone(),
            &mock_gpio,
        )
        .unwrap();
    }
}
