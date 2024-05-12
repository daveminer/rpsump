use tokio::{
    runtime::Handle,
    sync::mpsc::Receiver,
    time::{sleep, Duration},
};

use super::{control::SharedOutputPin, gpio::Level};

#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    SumpEmpty,
    SumpFull,
    IrrigatorEmpty,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Signal {
    pub message: Message,
    pub level: Level,
}

/// Set the Controls based on messages received from Sensors
///
/// # Arguments
///
/// * `rx` - The channel to receive messages from
/// * `handle`           - The tokio runtime handle
/// * `irrigator`        - The Irrigator instance
/// * `sump`             - The Sump instance
/// * `sump_empty_delay` - The delay to wait before turning off the sump pump;
///                        this is to clear the hose of water.
///
pub fn listen(
    mut rx: Receiver<Signal>,
    handle: Handle,
    irrigator_pump_pin: SharedOutputPin,
    sump_pump_pin: SharedOutputPin,
    sump_empty_delay: u64,
) {
    handle.spawn(async move {
        while let Some(signal) = rx.recv().await {
            // TODO: check levels
            match signal.message {
                Message::SumpEmpty => {
                    sleep(Duration::from_secs(sump_empty_delay)).await;

                    let pin = sump_pump_pin.clone();
                    let mut lock = pin.lock().await;
                    lock.off();
                }
                Message::SumpFull => {
                    let pin = sump_pump_pin.clone();

                    let mut lock = pin.lock().await;
                    lock.on();
                }
                Message::IrrigatorEmpty => {
                    let pin = irrigator_pump_pin.clone();
                    let mut lock = pin.lock().await;
                    lock.off();
                }
            }
        }
    });
}

//#[cfg(test)]
//mod tests {
//    use std::sync::Arc;

//    use rstest::*;
// use tokio::sync::mpsc;

// use crate::{
//     hydro::{
//         gpio::{Level, MockGpio},
//         irrigator::Irrigator,
//         signal::{listen, Message, Signal},
//         sump::Sump,
//     },
//     test_fixtures::{
//         gpio::{mock_irrigation_pump, mock_sump_pump},
//         settings::SETTINGS,
//     },
// };

// #[rstest]
// #[tokio::test]
// async fn test_listen() {
// let (tx, rx) = mpsc::channel(32);
// let handle = tokio::runtime::Handle::current();

// let mock_gpio = MockGpio::new();
// let mock_gpio = mock_irrigation_pump(mock_gpio, false, Level::High, false, None);
// let mock_gpio = mock_sump_pump(mock_gpio, false, false, false);

// let irrigator =
//     Irrigator::new(&SETTINGS.hydro.irrigation, &tx, handle.clone(), &mock_gpio).unwrap();
// let sump = Sump::new(&SETTINGS.hydro.sump, &tx, handle.clone(), &mock_gpio).unwrap();
// let sump_empty_delay = 1;

// listen(
//     rx,
//     handle,
//     irrigator.pump.pin.clone(),
//     sump.pump.pin.clone(),
//     sump_empty_delay,
// );

//tx.send(Message::SumpEmpty).await.unwrap();

// let signal = Signal {
//     message: Message::IrrigatorEmpty,
//     level: Level::High,
// };
// tx.send(signal).await.unwrap();

//let irrigator_pin = irrigator.pump.pin.clone();
//let irrigator_lock = irrigator_pin.lock().unwrap();

// TODO: assertions
//}
//}
