use rppal::gpio::Level;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::tungstenite::protocol::Message;

use crate::sump::Sump;

pub struct Board {
    pub sump: Arc<Mutex<Sump>>,
}

impl Board {
    pub fn start(tx: Sender<Message>) -> Board {
        let sump = Arc::new(Mutex::new(
            Sump::new(tx).expect("Could not create sump object"),
        ));

        Board { sump }
    }

    pub fn update_high_sump_sensor(self, state: Level) {
        let mut sump_state = self.sump.lock().unwrap();

        sump_state.high_sensor_state = state;
        // Check new sump general state
    }

    pub fn report(&self) -> String {
        let sump_state = self.sump.lock().unwrap();
        format!("Sump pump state: {:?}", sump_state.high_sensor)
    }
}
