use anyhow::{anyhow, Error};
use tokio::sync::mpsc::Receiver;
use tokio_tungstenite::tungstenite::protocol::Message;

pub async fn listen(mut rx: Receiver<Message>) -> Result<(), Error> {
    loop {
        match rx.recv().await {
            Some(msg) => println!("MSG: {:?}", msg),
            None => return Err(anyhow!("Empty message in channel")),
        }
    }
}
