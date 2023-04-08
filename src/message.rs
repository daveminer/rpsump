use anyhow::{anyhow, Error};
use rppal::gpio::Level;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::Receiver;
use tokio_tungstenite::tungstenite::protocol::Message;

use crate::sump::SensorState;

#[derive(Serialize, Deserialize)]
struct SumpSensorMessage {
    component: String,
    signal: String,
    level: String,
}

pub async fn listen(
    sensor_state: Arc<Mutex<SensorState>>,
    mut rx: Receiver<Message>,
) -> Result<(), Error> {
    loop {
        match rx.recv().await {
            Some(msg) => {
                let deserialized: SumpSensorMessage =
                    serde_json::from_str(&msg.to_string()).unwrap();

                let new_lvl = if deserialized.level == "high" {
                    Level::High
                } else {
                    Level::Low
                };

                let mut sensor = sensor_state.lock().unwrap();

                if sensor.high_sensor_state != new_lvl {
                    sensor.high_sensor_state = new_lvl;
                }
                println!("MSG: {:?}", msg);
            }
            None => println!("Empty message"), //return Err(anyhow!("Empty message in channel")),
        }
    }
}
