use std::sync::Arc;

use tokio::{
    runtime::Handle,
    sync::{mpsc::Receiver, Notify},
    time::{sleep, Duration},
};

use super::control::SharedOutputPin;
#[derive(Clone, Debug)]
pub enum Message {
    SumpEmpty,
    SumpFull,
    IrrigatorEmpty,
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
    mut rx: Receiver<Message>,
    handle: Handle,
    irrigator_pump_pin: SharedOutputPin,
    irrigator_notifier: Option<Arc<Notify>>,
    sump_pump_pin: SharedOutputPin,
    sump_notifier: Option<Arc<Notify>>,
    sump_empty_delay: u64,
) {
    handle.spawn(async move {
        let inote = irrigator_notifier;
        let snote = sump_notifier;
        while let Some(message) = rx.recv().await {
            match message {
                Message::SumpEmpty => {
                    sleep(Duration::from_secs(sump_empty_delay)).await;

                    let pin = sump_pump_pin.clone();
                    let _ = tokio::task::spawn_blocking(move || {
                        let mut lock = pin.lock().unwrap();
                        lock.off();
                    });

                    let snote = snote.clone();
                    if snote.is_some() {
                        snote.unwrap().notify_one();
                    }
                }
                Message::SumpFull => {
                    let pin = sump_pump_pin.clone();
                    let _ = tokio::task::spawn_blocking(move || {
                        let mut lock = pin.lock().unwrap();
                        lock.on();
                    });

                    let snote = snote.clone();
                    if snote.is_some() {
                        snote.unwrap().notify_one();
                    }
                }
                Message::IrrigatorEmpty => {
                    let pin = irrigator_pump_pin.clone();
                    let _ = tokio::task::spawn_blocking(move || {
                        let mut lock = pin.lock().unwrap();
                        lock.off();
                    });

                    let inote = inote.clone();
                    if inote.is_some() {
                        inote.unwrap().notify_one();
                    }
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rstest::*;
    use tokio::sync::{mpsc, Notify};

    use crate::{
        hydro::{
            gpio::{Gpio, MockGpio},
            irrigator::Irrigator,
            signal::{listen, Message},
            sump::Sump,
        },
        test_fixtures::{
            gpio::{mock_irrigation_pump, mock_sump_pump},
            settings::SETTINGS,
        },
    };

    #[rstest]
    #[tokio::test]
    async fn test_listen() {
        let (tx, rx) = mpsc::channel(32);
        let handle = tokio::runtime::Handle::current();

        let mock_gpio = MockGpio::new();
        let mock_gpio = mock_irrigation_pump(mock_gpio, false, false, None);
        let mock_gpio: Box<dyn Gpio> = Box::new(mock_sump_pump(mock_gpio, false, false, false));

        let irrigator = Irrigator::new(&SETTINGS.hydro.irrigation, &tx, &mock_gpio).unwrap();
        let sump = Sump::new(&SETTINGS.hydro.sump, &tx, &mock_gpio).unwrap();
        let sump_empty_delay = 1;

        let irrigator_notify = Arc::new(Notify::new());
        let sump_notify = Arc::new(Notify::new());

        listen(
            rx,
            handle,
            irrigator.pump.pin.clone(),
            Some(irrigator_notify.clone()),
            sump.pump.pin.clone(),
            Some(sump_notify.clone()),
            sump_empty_delay,
        );

        //tx.send(Message::SumpEmpty).await.unwrap();

        tx.send(Message::IrrigatorEmpty).await.unwrap();

        // Wait for the state changes to occur
        irrigator_notify.notified().await;
        //sump_notify.notified().await;

        //tx.send(Message::SumpFull).await.unwrap();
        //sump_notify.notified().await;

        //let irrigator_pin = irrigator.pump.pin.clone();
        //let irrigator_lock = irrigator_pin.lock().unwrap();

        // TODO: assertions
    }
}
