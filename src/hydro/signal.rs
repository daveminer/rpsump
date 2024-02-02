use tokio::{
    runtime::Handle,
    sync::mpsc::Receiver,
    time::{sleep, Duration},
};

use crate::hydro::{irrigator::Irrigator, sump::Sump};

#[derive(Clone, Debug)]
pub enum Message {
    SumpEmpty,
    SumpFull,
    IrrigatorEmpty,
}

pub fn listen(
    mut rx: Receiver<Message>,
    handle: Handle,
    irrigator: Irrigator,
    sump: Sump,
    sump_empty_delay: u64,
) {
    handle.spawn(async move {
        while let Some(message) = rx.recv().await {
            match message {
                Message::SumpEmpty => {
                    sleep(Duration::from_secs(sump_empty_delay)).await;
                    let mut lock = sump.pump.lock().await;
                    lock.set_low();
                }
                Message::SumpFull => {
                    let mut lock = sump.pump.lock().await;
                    lock.set_high();
                }
                Message::IrrigatorEmpty => {
                    let mut lock = irrigator.pump.lock().await;
                    lock.set_low();
                }
            }
        }
    });
}
