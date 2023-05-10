use anyhow::Error;
use futures::executor::block_on;
use rppal::gpio::{Gpio, InputPin, Level, OutputPin, Trigger};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::sync::{Arc, Mutex};
use std::{thread, time::Duration};

use crate::models::sump_event::SumpEvent;
use crate::{config::SumpConfig, database::DbPool};

// Manages the physical I/O devices
#[derive(Clone, Debug)]
pub struct Sump {
    pub db_pool: DbPool,
    pub high_sensor_pin: Arc<Mutex<InputPin>>,
    pub low_sensor_pin: Arc<Mutex<InputPin>>,
    pub pump_control_pin: Arc<Mutex<OutputPin>>,
    pub sensor_state: Arc<Mutex<PinState>>,
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

#[derive(Clone, Copy, Debug)]
enum Sensor {
    Low = 0,
    High = 1,
}

impl Sump {
    // Creates a new sump struct with sensors and their state.
    pub fn new(db_pool: DbPool, config: &SumpConfig) -> Result<Self, Error> {
        // create the GPIO pins
        let gpio = Gpio::new()?;
        let mut high_sensor_pin = gpio.get(config.high_sensor_pin)?.into_input_pullup();
        let mut low_sensor_pin = gpio.get(config.low_sensor_pin)?.into_input_pullup();
        let pump_control_pin: Arc<Mutex<OutputPin>> = Arc::from(Mutex::new(
            gpio.get(config.pump_control_pin)?.into_output_low(),
        ));

        // Read initial state of inputs
        let sensor_state = Arc::from(Mutex::new(PinState {
            high_sensor: high_sensor_pin.read(),
            low_sensor: low_sensor_pin.read(),
        }));

        // Set up interrupts
        Self::water_sensor_interrupt(
            db_pool.clone(),
            &mut high_sensor_pin,
            Arc::clone(&pump_control_pin),
            Arc::clone(&sensor_state),
            Sensor::High,
            config.pump_shutoff_delay,
        );

        Self::water_sensor_interrupt(
            db_pool.clone(),
            &mut low_sensor_pin,
            Arc::clone(&pump_control_pin),
            Arc::clone(&sensor_state),
            Sensor::Low,
            config.pump_shutoff_delay,
        );

        Ok(Sump {
            db_pool,
            high_sensor_pin: Arc::from(Mutex::new(high_sensor_pin)),
            low_sensor_pin: Arc::from(Mutex::new(low_sensor_pin)),
            pump_control_pin,
            sensor_state,
        })
    }

    // Read the current state of the sensors
    pub fn sensors(&self) -> PinState {
        *self.sensor_state.lock().unwrap()
    }

    fn water_sensor_interrupt(
        db: DbPool,
        pin: &mut InputPin,
        pump_control_pin: Arc<Mutex<OutputPin>>,
        sensor_state: Arc<Mutex<PinState>>,
        sensor_name: Sensor,
        pump_shutoff_delay: u64,
    ) {
        pin.set_async_interrupt(Trigger::Both, move |level| {
            Self::water_sensor_state_change_callback(
                sensor_name.clone(),
                db.clone(),
                level,
                Arc::clone(&pump_control_pin),
                Arc::clone(&sensor_state),
                pump_shutoff_delay,
            )
        })
        .expect("Could not not listen on high water level sump pin");
    }

    // Call this when a sensor change event happens.
    fn water_sensor_state_change_callback(
        triggered_sensor: Sensor,
        db: DbPool,
        level: Level,
        pump_control_pin: Arc<Mutex<OutputPin>>,
        sensor_state: Arc<Mutex<PinState>>,
        pump_shutoff_delay: u64,
    ) {
        let mut control = pump_control_pin.lock().unwrap();
        let mut sensors = sensor_state.lock().unwrap();

        // Turn the sump pump motor on or off
        match triggered_sensor {
            Sensor::High => {
                if level == Level::High {
                    control.set_high();

                    // TODO: set a time limit for this
                    let event_future = async {
                        match SumpEvent::create("kind".to_string(), "info".to_string(), db).await {
                            Ok(_) => {}
                            Err(e) => {
                                // TODO: log this
                                println!("Error creating sump event: {}", e);
                            }
                        }
                    };

                    block_on(event_future);
                }

                sensors.high_sensor = level;
            }
            Sensor::Low => {
                if level != Level::High {
                    // Let the pump run a bit longer or the sensor might remain high.
                    if pump_shutoff_delay > 0 {
                        thread::sleep(Duration::from_millis(pump_shutoff_delay as u64 * 1000));
                    }

                    control.set_low();

                    let event_future = async {
                        match SumpEvent::create("kind".to_string(), "info".to_string(), db).await {
                            Ok(_) => {}
                            Err(e) => {
                                // TODO: log this
                                println!("Error creating sump event: {}", e);
                            }
                        }
                    };

                    block_on(event_future);
                }
                sensors.low_sensor = level;
            }
        }
    }
}
