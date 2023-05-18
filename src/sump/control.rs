use rppal::gpio::{Level, OutputPin};

use crate::database::DbPool;
use crate::models::sump_event::SumpEvent;
use crate::sump::sensor::PinState;
use std::sync::{Arc, Mutex};
use std::{thread, time::Duration};

pub async fn update_high_sensor(
    level: Level,
    pump_control_pin: Arc<Mutex<OutputPin>>,
    sensor_state: Arc<Mutex<PinState>>,
    db: DbPool,
) {
    println!("UPDATING HIGH SENSOR: {:?}", level);
    // Turn the pump on
    if level == Level::High {
        let mut pin = pump_control_pin.lock().unwrap();
        pin.set_high();

        SumpEvent::create("pump on".to_string(), "reservoir full".to_string(), db)
            .await
            .unwrap();
    }

    let mut sensors = sensor_state.lock().unwrap();

    sensors.high_sensor = level;
}

pub async fn update_low_sensor(
    level: Level,
    pump_control_pin: Arc<Mutex<OutputPin>>,
    sensor_state: Arc<Mutex<PinState>>,
    delay: u64,
    db: DbPool,
) {
    println!("UPDATING LOW SENSOR: {:?}", level);
    // Turn the pump off
    if level == Level::Low {
        if delay > 0 {
            thread::sleep(Duration::from_millis(delay as u64 * 1000));
        }

        let mut pin = pump_control_pin.lock().unwrap();
        pin.set_low();

        SumpEvent::create("pump off".to_string(), "reservoir empty".to_string(), db)
            .await
            .unwrap();
    }

    let mut sensors = sensor_state.lock().unwrap();

    sensors.low_sensor = level;
}
