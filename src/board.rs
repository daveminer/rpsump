use tokio::sync::mpsc::Sender;
use tokio_tungstenite::tungstenite::protocol::Message;

use crate::sump::Sump;

pub struct Board {
    pub sump: Sump,
}

impl Board {
    pub fn start(tx: Sender<Message>) -> Board {
        let mut sump = Sump::new(tx).expect("Could not create sump object");

        sump.listen();
        Board { sump }
    }

    // pub fn update_high_sump_sensor(self, state: Level) {
    //     let mut sump_state = self.sump.lock().unwrap();

    //     sump_state.high_sensor_state = state;
    //     // Check new sump general state
    //     drop(sump_state);
    // }

    pub fn report(&self) -> String {
        let sump = &self.sump;

        format!("Sump pump state: {:?}", sump.sensors())
    }
}
