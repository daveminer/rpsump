use std::sync::Arc;

use tokio::{
    runtime::Handle,
    sync::{mpsc::Receiver, Notify},
    time::{sleep, Duration},
};

use crate::hydro::{irrigator::Irrigator, sump::Sump};
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
    irrigator: Irrigator,
    irrigator_notifier: Option<Arc<Notify>>,
    sump: Sump,
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

                    let pin = sump.pump.pin.clone();
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
                    let pin = sump.pump.pin.clone();
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
                    let pin = irrigator.pump.pin.clone();
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
        config::HydroConfig,
        hydro::{
            irrigator::Irrigator,
            signal::{listen, Message},
            sump::Sump,
        },
        test_fixtures::{gpio::mock_gpio_get, hydro::hydro_config},
    };

    #[rstest]
    #[tokio::test]
    async fn test_listen(#[from(hydro_config)] hydro_config: HydroConfig) {
        let (tx, rx) = mpsc::channel(32);
        let handle = tokio::runtime::Handle::current();
        let mock_gpio = mock_gpio_get(vec![6, 7, 8, 10, 12, 13, 14, 15, 16]);
        let irrigator = Irrigator::new(&hydro_config.irrigation, &tx, &mock_gpio).unwrap();
        let sump = Sump::new(&hydro_config.sump, &tx, &mock_gpio).unwrap();
        let sump_empty_delay = 1;

        let irrigator_notify = Arc::new(Notify::new());
        let sump_notify = Arc::new(Notify::new());

        listen(
            rx,
            handle,
            irrigator.clone(),
            Some(irrigator_notify.clone()),
            sump.clone(),
            Some(sump_notify.clone()),
            sump_empty_delay,
        );

        tx.send(Message::SumpEmpty).await.unwrap();

        tx.send(Message::IrrigatorEmpty).await.unwrap();

        // Wait for the state changes to occur
        irrigator_notify.notified().await;
        sump_notify.notified().await;

        tx.send(Message::SumpFull).await.unwrap();
        sump_notify.notified().await;

        let irrigator_pin = irrigator.pump.pin.clone();
        let irrigator_lock = irrigator_pin.lock().unwrap();

        //let sump_pin = sump.pump.pin.clone();
        //let sump_lock = sump_pin.lock().unwrap();

        assert_eq!(irrigator_lock.is_off(), true);
        // TODO: Fix test for block_on
        //assert_eq!(sump_lock.is_on(), true);
    }
}
