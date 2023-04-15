use anyhow::Error;
use rppal::gpio::{Gpio, InputPin, Level, OutputPin, Trigger};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::sync::{Arc, Mutex};

use crate::database::Database;

// GPIO uses BCM pin numbering.
const HIGH_SENSOR_PIN: u8 = 18; // GPIO #18 == Pin #12
const LOW_SENSOR_PIN: u8 = 24; // GPIO #24 == Pin #18
const PUMP_CONTROL_PIN: u8 = 14; // GPIO #14 == Pin #8

// Manages the physical I/O devices
#[derive(Debug)]
pub struct Sump {
    pub db: Database,
    pub high_sensor_pin: InputPin,
    pub low_sensor_pin: InputPin,
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
    pub fn new(db: Database) -> Result<Self, Error> {
        // create the GPIO pins
        let gpio = Gpio::new()?;
        let mut high_sensor_pin = gpio.get(HIGH_SENSOR_PIN)?.into_input_pullup();
        let mut low_sensor_pin = gpio.get(LOW_SENSOR_PIN)?.into_input_pullup();
        let pump_control_pin: Arc<Mutex<OutputPin>> =
            Arc::from(Mutex::new(gpio.get(PUMP_CONTROL_PIN)?.into_output_low()));

        // Read initial state of inputs
        let sensor_state = Arc::from(Mutex::new(PinState {
            high_sensor: high_sensor_pin.read(),
            low_sensor: low_sensor_pin.read(),
        }));

        // Set up interrupts
        Self::water_sensor_interrupt(
            &mut high_sensor_pin,
            Arc::clone(&pump_control_pin),
            Arc::clone(&sensor_state),
            Sensor::High,
        );

        Self::water_sensor_interrupt(
            &mut low_sensor_pin,
            Arc::clone(&pump_control_pin),
            Arc::clone(&sensor_state),
            Sensor::Low,
        );

        Ok(Sump {
            db,
            high_sensor_pin,
            low_sensor_pin,
            pump_control_pin,
            sensor_state,
        })
    }

    // Read the current state of the sensors
    pub fn sensors(&self) -> PinState {
        *self.sensor_state.lock().unwrap()
    }

    fn water_sensor_interrupt(
        pin: &mut InputPin,
        pump_control_pin: Arc<Mutex<OutputPin>>,
        sensor_state: Arc<Mutex<PinState>>,
        sensor_name: Sensor,
    ) {
        pin.set_async_interrupt(Trigger::Both, move |level| {
            Self::water_sensor_state_change_callback(
                sensor_name.clone(),
                level,
                Arc::clone(&pump_control_pin),
                Arc::clone(&sensor_state),
            )
        })
        .expect("Could not not listen on high water level sump pin");
    }

    // Call this when a sensor change event happens.
    fn water_sensor_state_change_callback(
        triggered_sensor: Sensor,
        level: Level,
        pump_control_pin: Arc<Mutex<OutputPin>>,
        sensor_state: Arc<Mutex<PinState>>,
    ) {
        let mut control = pump_control_pin.lock().unwrap();
        let mut sensors = sensor_state.lock().unwrap();

        // Turn the sump pump motor on or off
        match triggered_sensor {
            Sensor::High => {
                if level == Level::High {
                    control.set_high();
                }

                sensors.high_sensor = level;
            }
            Sensor::Low => {
                if level == Level::High {
                    // Start a timer (5 min) to clear a non-full container in a
                    // timely way.
                } else {
                    control.set_low();
                }

                sensors.low_sensor = level;
            }
        }
    }
}
