use rppal::gpio::{Level, OutputPin};
use std::sync::{Arc, Mutex};
use std::{thread, time::Duration};

use crate::database::DbPool;
use crate::models::sump_event::SumpEvent;
use crate::sump::sensor::PinState;

/// Applies a state change to the high sensor by settting the pin level, creating a database event,
/// and updating the sensor state.
pub async fn update_high_sensor(
    level: Level,
    pump_control_pin: Arc<Mutex<OutputPin>>,
    sensor_state: Arc<Mutex<PinState>>,
    db: DbPool,
) {
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

/// Applies a state change to the low sensor similar to the high sensor. The difference is that the
/// low sensor accepts a delay that allows the pump to run long to lower the water level enough to
/// prevent signal bouncing.
pub async fn update_low_sensor(
    level: Level,
    pump_control_pin: Arc<Mutex<OutputPin>>,
    sensor_state: Arc<Mutex<PinState>>,
    delay: u64,
    db: DbPool,
) {
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
