use actix_web::rt::Runtime;
use anyhow::Error;
use std::{
    process::Command,
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc::Sender;

use crate::{
    config::IrrigationConfig,
    hydro::{
        gpio::{Gpio, Trigger},
        sensor::Sensor,
        Control,
    },
};

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
    pub fn new<G>(
        config: &IrrigationConfig,
        tx: &Sender<Command>,
        rt: Arc<Mutex<Runtime>>,
        gpio: &G,
    ) -> Result<Self, Error>
    where
        G: Gpio,
    {
        let pump = Control::new("Irrigation Pump".to_string(), config.pump_control_pin, gpio)?;

        let low_sensor = Sensor::new(
            "Irrigator Empty".to_string(),
            config.low_sensor_pin,
            gpio,
            Trigger::Both,
            rt,
            tx,
            0,
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
    use std::sync::{Arc, Mutex};

    use actix_web::rt::Runtime;

    use crate::{
        config::IrrigationConfig,
        hydro::{
            gpio::{stub::pin, Level, MockGpio},
            message::Message,
        },
    };

    use super::Irrigator;

    #[test]
    fn test_new() {
        let mpsc = Message::init();

        // let mut mock_db_pool = MockDbPool::new();
        // mock_db_pool
        //     .expect_get_conn()
        //     .returning(|| Ok(MockDbConn::new())); // Replace with your mock connection

        let mut mock_gpio = MockGpio::new();
        mock_gpio.expect_get().times(6).returning(|_| {
            Ok(Box::new(pin::PinStub {
                index: 0,
                level: Level::Low,
            }))
        });

        let _irrigator: Irrigator = Irrigator::new(
            &IrrigationConfig {
                enabled: true,
                low_sensor_pin: 1,
                max_seconds_runtime: 2,
                process_frequency_ms: 1000,
                pump_control_pin: 2,
                valve_1_control_pin: 3,
                valve_2_control_pin: 4,
                valve_3_control_pin: 5,
                valve_4_control_pin: 6,
            },
            &mpsc.tx,
            Arc::from(Mutex::new(Runtime::new().unwrap())),
            &mock_gpio,
        )
        .unwrap();
    }
}
