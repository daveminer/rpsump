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

    pub fn report(&self) -> String {
        let sump = &self.sump;

        format!("Sump pump state: {:?}", sump.sensors())
    }
}
