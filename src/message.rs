use anyhow::{anyhow, Error};
use rppal::gpio::Level;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::Receiver;
use tokio_tungstenite::tungstenite::protocol::Message;

use crate::sump::SensorState;

#[derive(Serialize, Deserialize, Debug)]
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
                let deserialized: SumpSensorMessage = serde_json::from_str(&msg.to_string())?;

                let level = if deserialized.level == "high" {
                    Level::High
                } else {
                    Level::Low
                };

                let mut sensor = sensor_state.lock().unwrap();

                if sensor.high_sensor_state != level {
                    sensor.high_sensor_state = level;
                }
            }
            None => return Err(anyhow!("Channel has been closed.")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::sync::mpsc::channel;

    fn new_sump_sensor_message() -> SumpSensorMessage {
        SumpSensorMessage {
            component: "test_component".to_string(),
            signal: "test_signal".to_string(),
            level: "high".to_string(),
        }
    }

    #[tokio::test]
    async fn test_listen() {
        // Setup
        let (tx, rx) = channel(10);
        let sensor_state = Arc::new(Mutex::new(SensorState {
            high_sensor_state: Level::Low,
        }));

        let json_message = serde_json::to_string(&new_sump_sensor_message()).unwrap();
        let websocket_message = Message::Text(json_message.clone());

        let sensor_state_clone = Arc::clone(&sensor_state);

        // Test
        let listen_handle = tokio::spawn(async move {
            listen(sensor_state_clone, rx).await.unwrap();
        });
        tx.send(websocket_message).await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
        listen_handle.abort();

        // Verify
        let sensor = sensor_state.lock().unwrap();
        assert_eq!(sensor.high_sensor_state, Level::High);
    }

    #[test]
    fn test_sump_sensor_message_serialization() {
        let message = new_sump_sensor_message();
        let expected_json =
            r#"{"component":"test_component","signal":"test_signal","level":"high"}"#;
        let serialized_json = serde_json::to_string(&message).unwrap();
        assert_eq!(serialized_json, expected_json);

        let deserialized_message: SumpSensorMessage = serde_json::from_str(expected_json).unwrap();
        assert_eq!(deserialized_message.component, message.component);
        assert_eq!(deserialized_message.signal, message.signal);
        assert_eq!(deserialized_message.level, message.level);
    }
}
