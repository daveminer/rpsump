use std::process::Command;
use tokio::sync::mpsc;

pub struct Message {
    pub tx: mpsc::Sender<Command>,
    pub rx: mpsc::Receiver<Command>,
}

impl Message {
    pub fn init() -> Self {
        let (tx, rx) = mpsc::channel(32);
        Self { tx, rx }
    }
}
