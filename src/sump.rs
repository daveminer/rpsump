use anyhow::Error;
use rppal::gpio::{Gpio, InputPin, Level, Trigger};

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
    //low_sensor: InputPin,
    pub tx: Sender<Message>,
}

// Tracks the level of the sensor pins. It's intended for the fields of this
// struct to be read as an atomic unit to determine what the state of the pump
// should be.
#[derive(Clone, Copy, Debug)]
pub struct SensorState {
    pub high_sensor_state: Level,
}

impl Sump {
    // Creates a new sump struct with sensors and their state.
    pub fn new(tx: Sender<Message>) -> Result<Sump, Error> {
        let gpio = Gpio::new()?;

        let high_sensor = gpio.get(HIGH_SENSOR_PIN)?.into_input_pullup();
        //let mut low_sensor = gpio.get(low_pin)?.into_input_pullup();

        //Self::listen_on_input_pin(&mut high_sensor, "high sensor".to_string(), tx.clone());
        //Self::listen_on_input_pin(&mut low_sensor, "low sensor".to_string(), tx.clone());

        let sensor_state = Arc::new(Mutex::new(SensorState {
            high_sensor_state: high_sensor.read(),
        }));

        Ok(Sump {
            high_sensor,
            sensor_state,
            //low_sensor,
            tx,
        })
    }

    pub fn listen(&mut self) {
        let tx = self.tx.clone();

        self.high_sensor
            .set_async_interrupt(Trigger::Both, move |level| {
                println!("INTERRUPT");
                let msg = Message::Text(format!("High sensor to {}", level));
                match tx.blocking_send(msg) {
                    Ok(_) => (), //self.update_high_sump_sensor(level),
                    Err(e) => println!("Error on message tx: {:?}", e),
                };
                //self.sump_signal_received(level);
            })
            .expect("Could not not listen on sump pin")
    }

    fn sump_signal_received(self, level: Level) {
        println!("CALLBACK");

        let msg = Message::Text(format!("{:?} to {}", self.high_sensor, level));
        match self.tx.blocking_send(msg) {
            Ok(_) => self.update_high_sump_sensor(level),
            Err(e) => println!("Error on message tx: {:?}", e),
        };
    }

    // fn listen_on_input_pin(pin: &mut InputPin, sensor: InputPin, tx: Sender<Message>) {
    //     pin.set_async_interrupt(Trigger::Both, move |level| {
    //         println!("INTERRUPT");
    //         sump_signal_received(level, sensor, tx.clone());
    //     })
    //     .expect("Could not not listen on sump pin");
    // }

    pub fn sensors(&self) -> SensorState {
        let sensor_state = self.sensor_state.lock().unwrap();
        let sensor_reading = *sensor_state;

        sensor_reading
    }

    pub fn update_high_sump_sensor(self, state: Level) {
        let mut sensor_state = self.sensor_state.lock().unwrap();

        sensor_state.high_sensor_state = state;
        // Check new sump general state
    }
}
