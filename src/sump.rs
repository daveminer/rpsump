use anyhow::Error;
use rppal::gpio::{Gpio, InputPin, Level, OutputPin, Trigger};
use serde_json::json;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::tungstenite::protocol::Message;

// Gpio uses BCM pin numbering. BCM GPIO 23 is tied to physical pin 16.
const HIGH_SENSOR_PIN: u8 = 14; // GPIO #14 == Pin #8
const LOW_SENSOR_PIN: u8 = 15; // GPIO #15 == Pin #10
const PUMP_CONTROL_PIN: u8 = 18; // GPIO #18 == Pin #12

// Manages the physical I/O devices
#[derive(Debug)]
pub struct Sump {
    pub high_sensor_pin: InputPin,
    pub low_sensor_pin: InputPin,
    pub pump_control_pin: Arc<Mutex<OutputPin>>,
    pub sensor_state: Arc<Mutex<PinState>>,
    pub tx: Sender<Message>,
}

// Tracks the level of the sensor pins. It's intended for the fields of this
// struct to be read as an atomic unit to determine what the state of the pump
// should be.
#[derive(Clone, Copy, Debug)]
pub struct PinState {
    pub high_sensor: Level,
    pub low_sensor: Level,
}

impl Sump {
    // Creates a new sump struct with sensors and their state.
    pub fn new(tx: Sender<Message>) -> Result<Sump, Error> {
        let gpio = Gpio::new()?;

        let high_sensor_pin = gpio.get(HIGH_SENSOR_PIN)?.into_input_pullup();
        let low_sensor_pin = gpio.get(LOW_SENSOR_PIN)?.into_input_pullup();
        let pump_control_pin: Arc<Mutex<OutputPin>> =
            Arc::from(Mutex::new(gpio.get(PUMP_CONTROL_PIN)?.into_output_low()));

        let sensor_state = Arc::from(Mutex::new(PinState {
            high_sensor: high_sensor_pin.read(),
            low_sensor: low_sensor_pin.read(),
        }));

        Ok(Sump {
            high_sensor_pin,
            low_sensor_pin,
            pump_control_pin,
            sensor_state,
            tx,
        })
    }

    // Starts a listener that will produce a channel message for each sensor event
    pub fn listen(&mut self) {
        Self::water_sensor_interrupt(
            &mut self.high_sensor_pin,
            Arc::clone(&self.pump_control_pin),
            "high".to_string(),
            self.tx.clone(),
        );

        Self::water_sensor_interrupt(
            &mut self.low_sensor_pin,
            Arc::clone(&self.pump_control_pin),
            "low".to_string(),
            self.tx.clone(),
        );
    }

    // Read the current state of the sensors
    pub fn sensors(&self) -> PinState {
        *self.sensor_state.lock().unwrap()
    }

    fn water_sensor_interrupt(
        pin: &mut InputPin,
        pump_control_pin: Arc<Mutex<OutputPin>>,
        sensor_name: String,
        tx: Sender<Message>,
    ) {
        pin.set_async_interrupt(Trigger::Both, move |level| {
            Self::water_sensor_state_change_callback(
                sensor_name.clone(),
                level,
                Arc::clone(&pump_control_pin),
                &tx,
            )
        })
        .expect("Could not not listen on high water level sump pin");
    }

    // Call this when a sensor change event happens
    fn water_sensor_state_change_callback(
        sensor_name: String,
        level: Level,
        pump_control_pin: Arc<Mutex<OutputPin>>,
        tx: &Sender<Message>,
    ) {
        let msg = Self::water_level_change_message(sensor_name, level);

        let mut control = pump_control_pin.lock().unwrap();

        match level {
            Level::High => control.set_high(),
            Level::Low => control.set_low(),
        }

        match tx.blocking_send(msg) {
            Ok(_) => (),
            Err(e) => println!("Error on message tx: {:?}", e),
        };
    }

    // Build the channel message for a sensor change event
    fn water_level_change_message(sensor_name: String, level: Level) -> Message {
        let level_str = match level {
            Level::High => "high",
            Level::Low => "low",
        };

        let msg_body = json!({
            "component" : "sump pump",
            "signal": format!("Sump pump {} water sensor", sensor_name),
            "level": level_str
        });

        Message::Text(msg_body.to_string())
    }
}
