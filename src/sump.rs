use anyhow::Error;
use rppal::gpio::{Gpio, InputPin, Level, Trigger};
use serde_json::json;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::tungstenite::protocol::Message;

// Gpio uses BCM pin numbering. BCM GPIO 23 is tied to physical pin 16.
//const GPIO_LOW_SENSOR: u8 = 99;
const HIGH_SENSOR_PIN: u8 = 14; // GPIO #14 == Pin #8

// Manages the physical I/O devices
#[derive(Debug)]
pub struct Sump {
    pub high_sensor: InputPin,
    pub sensor_state: Arc<Mutex<SensorState>>,
    pub tx: Sender<Message>,
}

// Tracks the level of the sensor pins. It's intended for the fields of this
// struct to be read as an atomic unit to determine what the state of the pump
// should be.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SensorState {
    pub high_sensor_state: Level,
    pub low_sensor_state: Level,
}

impl Sump {
    // Creates a new sump struct with sensors and their state.
    pub fn new(tx: Sender<Message>) -> Result<Sump, Error> {
        let gpio = Gpio::new()?;

        let high_sensor = gpio.get(HIGH_SENSOR_PIN)?.into_input_pullup();

        let sensor_state = Arc::from(Mutex::new(SensorState {
            high_sensor_state: tokio::task::block_in_place(|| high_sensor.read()),
        }));

        Ok(Sump {
            high_sensor,
            sensor_state,
            tx,
        })
    }

    // Starts a listener that will produce a channel message for each sensor event
    pub fn listen(&mut self) {
        let tx = self.tx.clone();

        self.high_sensor
            .set_async_interrupt(Trigger::Both, move |level| {
                let level_str = match level {
                    Level::High => "high",
                    Level::Low => "low",
                };

                let msg_body = json!({
                    "component" : "sump pump",
                    "signal": "high water sensor",
                    "level": level_str
                });

                match tx.blocking_send(Message::Text(msg_body.to_string())) {
                    Ok(_) => (),
                    Err(e) => println!("Error on message tx: {:?}", e),
                };
            })
            .expect("Could not not listen on sump pin")
    }

    pub fn sensors(&self) -> SensorState {
        let sensor_state = self.sensor_state.lock().unwrap();
        let sensor_reading = *sensor_state;

        sensor_reading
    }
}
