use rppal::gpio::{Level, Trigger};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::runtime::Runtime;

use super::*;
use crate::database::DbPool;
use crate::sump::control::{update_high_sensor, update_low_sensor};
use crate::sump::debounce::SensorDebouncer;

// Tracks the level of the sensor pins. It's intended for the fields of this
// struct to be read as an atomic unit to determine what the state of the pump
// should be.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct PinState {
    #[serde(
        serialize_with = "serialize_level",
        deserialize_with = "deserialize_level"
    )]
    pub high_sensor: Level,
    #[serde(
        serialize_with = "serialize_level",
        deserialize_with = "deserialize_level"
    )]
    pub low_sensor: Level,
}

fn serialize_level<S>(level: &Level, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u8(*level as u8)
}

fn deserialize_level<'de, D>(deserializer: D) -> Result<Level, D::Error>
where
    D: Deserializer<'de>,
{
    let value = u8::deserialize(deserializer)?;
    match value {
        0 => Ok(Level::Low),
        1 => Ok(Level::High),
        _ => Err(serde::de::Error::custom("invalid Level value")),
    }
}

pub fn listen_to_high_sensor(
    high_sensor_pin: SharedInputPin,
    pump_control_pin: SharedOutputPin,
    sensor_state: SharedPinState,
    db: DbPool,
) {
    let debouncer: Arc<Mutex<Option<SensorDebouncer>>> = Arc::from(Mutex::new(None));
    let mut pin = high_sensor_pin.lock().unwrap();

    pin.set_async_interrupt(Trigger::Both, move |level| {
        let shared_deb = Arc::clone(&debouncer);
        let mut deb = shared_deb.lock().unwrap();

        if deb.is_some() {
            println!("UPDATING DEBOUNCER FOR {:?}", level);
            deb.as_mut().unwrap().reset_deadline(level);
            return;
        }

        println!("SETTING NEW HIGH DEBOUNCER FOR {:?}", level);
        let debouncer = SensorDebouncer::new(Duration::new(2, 0), level);
        *deb = Some(debouncer);

        let sleep = deb.as_ref().unwrap().sleep();

        let rt = Runtime::new().unwrap();
        rt.block_on(sleep);
        rt.block_on(update_high_sensor(
            level,
            Arc::clone(&pump_control_pin),
            Arc::clone(&sensor_state),
            db.clone(),
        ));
    })
    .expect("Could not not listen on high water level sump pin");
}

pub fn listen_to_low_sensor(
    low_sensor_pin: SharedInputPin,
    pump_control_pin: SharedOutputPin,
    sensor_state: SharedPinState,
    delay: u64,
    db: DbPool,
) {
    let debouncer: Arc<Mutex<Option<SensorDebouncer>>> = Arc::from(Mutex::new(None));
    let mut pin = low_sensor_pin.lock().unwrap();

    pin.set_async_interrupt(Trigger::Both, move |level| {
        let shared_deb = Arc::clone(&debouncer);
        let mut deb = shared_deb.lock().unwrap();

        if deb.is_some() {
            println!("UPDATING LOW DEBOUNCER FOR {:?}", level);
            deb.as_mut().unwrap().reset_deadline(level);
            return;
        }

        println!("SETTING NEW LOW DEBOUNCER FOR {:?}", level);
        let debouncer = SensorDebouncer::new(Duration::new(2, 0), level);
        *deb = Some(debouncer);

        let sleep = deb.as_ref().unwrap().sleep();

        let rt = Runtime::new().unwrap();
        rt.block_on(sleep);
        rt.block_on(update_low_sensor(
            level,
            Arc::clone(&pump_control_pin),
            Arc::clone(&sensor_state),
            delay,
            db.clone(),
        ));
    })
    .expect("Could not not listen on low water level sump pin");
}
