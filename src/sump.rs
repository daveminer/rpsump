use anyhow::Error;
use rppal::gpio::{Gpio, InputPin, Level, OutputPin, Trigger};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::sync::{Arc, Mutex};
use std::{
    thread,
    time::{Duration, Instant},
};
use tokio::runtime::Runtime;

use crate::debounce::SensorDebouncer;
use crate::models::sump_event::SumpEvent;
use crate::{config::SumpConfig, database::DbPool};

pub type SharedSensorDebouncer = Arc<Mutex<Option<SensorDebouncer>>>;
pub type SharedInputPin = Arc<Mutex<InputPin>>;
pub type SharedOutputPin = Arc<Mutex<OutputPin>>;
pub type SharedPinState = Arc<Mutex<PinState>>;

// Manages the physical I/O devices
#[derive(Clone, Debug)]
pub struct Sump {
    pub db_pool: DbPool,
    pub high_sensor_debounce: SharedSensorDebouncer,
    pub high_sensor_pin: SharedInputPin,
    pub low_sensor_debounce: SharedSensorDebouncer,
    pub low_sensor_pin: SharedInputPin,
    pub pump_control_pin: SharedOutputPin,
    pub sensor_state: SharedPinState,
}

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

impl Sump {
    // Creates a new sump struct with sensors and their state.
    pub fn new(db_pool: DbPool, config: &SumpConfig) -> Result<Self, Error> {
        // create the GPIO pins
        let gpio = Gpio::new()?;

        let high_sensor_pin_io = gpio.get(config.high_sensor_pin)?.into_input_pullup();
        let high_sensor_reading = high_sensor_pin_io.read();
        let high_sensor_pin = Arc::from(Mutex::new(high_sensor_pin_io));
        let high_debounce = Arc::from(Mutex::new(None));

        let low_sensor_pin_io = gpio.get(config.low_sensor_pin)?.into_input_pullup();
        let low_sensor_reading = low_sensor_pin_io.read();
        let low_sensor_pin = Arc::from(Mutex::new(low_sensor_pin_io));
        let low_debounce = Arc::from(Mutex::new(None));

        let pump_control_pin: Arc<Mutex<OutputPin>> = Arc::from(Mutex::new(
            gpio.get(config.pump_control_pin)?.into_output_low(),
        ));

        // Read initial state of inputs
        let sensor_state = Arc::from(Mutex::new(PinState {
            high_sensor: high_sensor_reading,
            low_sensor: low_sensor_reading,
        }));

        listen_to_high_sensor(
            Arc::clone(&high_sensor_pin),
            Arc::clone(&pump_control_pin),
            Arc::clone(&sensor_state),
            db_pool.clone(),
        );

        listen_to_low_sensor(
            Arc::clone(&low_sensor_pin),
            Arc::clone(&pump_control_pin),
            Arc::clone(&sensor_state),
            config.pump_shutoff_delay,
            db_pool.clone(),
        );

        Ok(Sump {
            db_pool,
            high_sensor_debounce: Arc::clone(&high_debounce),
            high_sensor_pin: Arc::clone(&high_sensor_pin),
            low_sensor_debounce: Arc::clone(&low_debounce),
            low_sensor_pin: Arc::clone(&low_sensor_pin),
            pump_control_pin: Arc::clone(&pump_control_pin),
            sensor_state: Arc::clone(&sensor_state),
        })
    }
}

pub fn spawn_reporting_thread(
    sensor_state: SharedPinState,
    interval_seconds: u64,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut start_time = Instant::now();
        let sensors = Arc::clone(&sensor_state);

        loop {
            // Report to console
            let sensor_reading = *sensors.lock().unwrap();
            println!("{:?}", sensor_reading);

            // Wait for N seconds
            let elapsed_time = start_time.elapsed();
            if elapsed_time < Duration::from_secs(interval_seconds) {
                thread::sleep(Duration::from_secs(interval_seconds) - elapsed_time);
            }
            start_time = Instant::now();
        }
    })
}

async fn update_high_sensor(
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

async fn update_low_sensor(
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
