use tokio::{
    runtime::Handle,
    sync::mpsc::Receiver,
    task::JoinHandle,
    time::{sleep, Duration},
};

use super::{control::SharedOutputPin, gpio::Level};

#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    SumpEmpty,
    SumpFull,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Signal {
    pub message: Message,
    pub level: Level,
}

/// Set the Controls based on messages received from Sensors.
///
/// # Arguments
///
/// * `rx` - The channel to receive messages from
/// * `handle` - The tokio runtime handle
/// * `sump_pump_pin` - The sump pump output pin
/// * `sump_empty_delay` - Delay before turning off the sump pump (to clear the hose)
/// * `max_pump_runtime` - Safety timer that forces the pump off after this duration
pub fn listen(
    mut rx: Receiver<Signal>,
    handle: Handle,
    sump_pump_pin: SharedOutputPin,
    sump_empty_delay: u64,
    max_pump_runtime: u64,
) {
    handle.spawn(async move {
        let mut sump_pump_timer: Option<JoinHandle<()>> = None;
        while let Some(signal) = rx.recv().await {
            match signal.message {
                Message::SumpEmpty => {
                    if let Some(handle) = sump_pump_timer.take() {
                        handle.abort();
                    }

                    sleep(Duration::from_secs(sump_empty_delay)).await;

                    let pin = sump_pump_pin.clone();
                    let mut lock = pin.lock().await;
                    lock.off();
                }
                Message::SumpFull => {
                    let pin = sump_pump_pin.clone();
                    let mut lock = pin.lock().await;
                    lock.on();

                    if let Some(handle) = sump_pump_timer.take() {
                        handle.abort();
                    }
                    let pin_clone = sump_pump_pin.clone();
                    sump_pump_timer = Some(tokio::spawn(async move {
                        tokio::time::sleep(Duration::from_secs(max_pump_runtime)).await;
                        let mut lock = pin_clone.lock().await;
                        lock.off();
                        tracing::warn!("Sump pump ran for too long, turning off with safety timer");
                    }));
                }
            }
        }
    });
}
