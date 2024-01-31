use anyhow::{anyhow, Error};
use std::{
    fmt::Debug,
    process::Command,
    sync::{Arc, Mutex},
};
use tokio::{runtime::Handle, sync::mpsc::Sender};

use crate::hydro::{
    debounce::Debouncer,
    gpio::{Gpio, InputPin, Trigger},
    Level,
};

/// Sensors will trigger async callbacks (which create a thread) on these
pub type SharedInputPin = Arc<Mutex<Box<dyn InputPin>>>;
/// Threads spawned from sensor state changes will share one of these per sensor
pub type SharedSensorDebouncer = Arc<Mutex<Option<Debouncer>>>;

pub mod high_sump;
pub mod low_irrigator;
pub mod low_sump;

#[derive(Clone, Debug)]
pub struct Sensor {
    pub level: Level,
    pub pin: SharedInputPin,
    pub debounce: SharedSensorDebouncer,
}

pub trait Input {
    fn is_high(&self) -> bool;
    fn is_low(&self) -> bool;
    fn level(&self) -> Level;
}

/// Represents a GPIO output for controlling stateful equipment. Water pump, etc.
/// Can be read synchronously or listened to for events.
///
/// # Arguments
///
/// * `name` - The name of the sensor for labelling
/// * `pin_number` - The GPIO pin number to listen to
/// * `gpio` - The GPIO implementation to use
/// * `trigger` - The trigger to listen for; rising, falling, or both
/// * `handle` - handler function to run when the trigger is detected
/// * `tx` - The channel to send commands to
/// * `delay` - The debounce delay in milliseconds, if any. Used to
///    leave the pump on momentarily after a low water level is detected.
impl Sensor {
    pub fn new<G>(
        name: String,
        pin_number: u8,
        gpio: &G,
        trigger: Trigger,
        handle: &Handle,
        tx: &Sender<Command>,
        delay: u64,
    ) -> Result<Self, Error>
    where
        G: Gpio,
    {
        let mut pin_io = gpio
            .get(pin_number)
            .map_err(|e| anyhow!(e))?
            .into_input_pullup();

        let _ = pin_io
            .set_async_interrupt(name.to_string(), trigger, handle.clone(), tx, delay)
            .map_err(|e| anyhow!(e.to_string()))?;

        Ok(Self {
            level: pin_io.read(),
            pin: Arc::from(Mutex::new(pin_io)),
            debounce: Arc::from(Mutex::new(None)),
        })
    }
}

impl Input for Sensor {
    fn is_high(&self) -> bool {
        self.level == Level::High
    }

    fn is_low(&self) -> bool {
        self.level == Level::Low
    }

    fn level(&self) -> Level {
        self.level
    }
}

// #[tracing::instrument(skip(db))]
// pub fn level_change_handler<Fut>(
//     level: Level,
//     handler: impl FnOnce(Level, Control, DbPool) -> Fut,
//     shared_debouncer: Arc<Mutex<Option<Debouncer>>>,
//     pump: Control,
//     db: DbPool,
//     rt: &mut Runtime,
// ) where
//     Fut: Future<Output = ()>,
// {
//     let deb = Arc::clone(&shared_debouncer);
//     let mut deb_lock = deb.lock().unwrap();

//     if deb_lock.is_some() {
//         deb_lock.as_mut().unwrap().reset_deadline(level);
//         return;
//     }

//     let sleep = deb_lock.as_ref().unwrap().sleep();

//     rt.block_on(async {
//         sleep.await;
//         *deb_lock = None;
//         drop(deb_lock);

//         handler(level, pump, db.clone()).await;
//     });
// }

// #[tracing::instrument(skip(db))]
// pub async fn handle_sensor_signal(action: Level, mut pump: Control, db: DbPool) {
//     if delay > 0 {
//         sleep(Duration::from_secs(delay as u64)).await;
//     }

//     match action {
//         Level::High => pump.on().await,
//         Level::Low => pump.off().await,
//         Level::Both => Ok(()),
//     };

//     tracing::info!("Sump pump turned {:?}.", action);

//     if let Err(e) = SumpEvent::create(format!("pump {:?}", action), "".to_string(), db).await {
//         tracing::error!(
//             target = module_path!(),
//             error = e.to_string(),
//             "Failed to create sump event for pump on"
//         );
//     };
// }

// #[tracing::instrument(skip(db))]
// pub fn listen(control: Control, sensor: Sensor, db: DbPool) -> Result<(), Error> {
//     let mut pin = sensor_pin
//         .lock()
//         .map_err(|e| anyhow!(e.to_string()))
//         .context("Could not get sensor pin.")?;

//     let debouncer: Arc<Mutex<Option<SensorDebouncer>>> = Arc::from(Mutex::new(None));

//     pin.set_async_interrupt(Trigger::Both, move |level| {
//         if let Err(e) = handle_interrupt(level, &debouncer, &control_pin, &sensor_state, &db) {
//             error_trace(e, "Error handling interrupt");
//         }
//     })
//     .context("Could not listen on sump pin.")?;

//     Ok(())
// pin.set_async_interrupt(Trigger::Both, move |level| {-> Result<(), Error>
//     let shared_deb = Arc::clone(&debouncer);
//     let mut deb = match shared_deb.lock() {
//         Ok(deb) => deb,
//         Err(e) => {
//             error_trace(anyhow!(e.to_string()), "Could not get sensor debouncer");
//             return;
//         }
//     };

//     // Debounce has already started; reset the deadline
//     if deb.is_some() {
//         match deb.as_mut() {
//             Some(debouncer) => debouncer.reset_deadline(level),
//             None => {
//                 error_trace(
//                     anyhow!("Sensor debouncer is None"),
//                     "Could not reset sensor debouncer deadline",
//                 );
//                 return;
//             }
//         };

//         return;
//     }

//     // Create a new debouncer
//     *deb = Some(SensorDebouncer::new(Duration::new(2, 0), level));

//     let sleep = deb.as_ref().unwrap().sleep();
//     let rt = Runtime::new().unwrap();
//     // Wait for the sleep period
//     rt.block_on(sleep);

//     // Update the sensor
//     rt.block_on(update_high_sensor(
//         level,
//         Arc::clone(&control_pin),
//         Arc::clone(&sensor_state),
//         db.clone(),
//     ));

//     *deb = None;
//     drop(deb);
// })
// .expect("Could not not listen on high water level sump pin.");
//}

// fn handle_interrupt(
//     level: Level,
//     debouncer: &Arc<Mutex<Option<Debouncer>>>,
//     control: Control,
//     db: &DbPool,
// ) -> Result<(), Error> {
//     let mut deb = debouncer
//         .lock()
//         .map_err(|e| anyhow!(e.to_string()))
//         .context("Could not acquire debouncer lock")?;

//     if let Some(ref mut debouncer) = *deb {
//         // Debounce has already started; reset the deadline
//         debouncer.reset_deadline(level);
//         return Ok(());
//     }

//     // Create a new debouncer
//     let new_debouncer = Debouncer::new(Duration::new(2, 0), level);
//     *deb = Some(new_debouncer);

//     // The `drop` is important to release the lock before awaiting
//     drop(deb);

//     // Perform the sleep and sensor update asynchronously
//     tokio::spawn(async move {
//         // Wait for the sleep period
//         tokio::time::sleep(Duration::new(2, 0)).await;

//         // Update the sensor
//         // if let Err(e) = update_high_sensor(
//         //     level,
//         //     Arc::clone(control_pin),
//         //     Arc::clone(sensor_state),
//         //     db.clone(),
//         // )
//         // .await
//         // {
//         //     error_trace(e, "Failed to update high sensor");
//         // }

//         // // Reset the debouncer
//         // let mut deb = match debouncer.lock() {
//         //     Ok(deb) => deb,
//         //     Err(e) => {
//         //         error_trace(anyhow!(e.to_string()), "Could not get sensor debouncer");
//         //         return;
//         //     }
//         // };
//         //let mut deb = debouncer
//         //    .lock()
//         //    .map_err(|e| anyhow!(e.to_string()))
//         //    .context("Could not acquire debouncer lock after sleep")?;
//         //*deb = None;
//     });

//     Ok(())
// }

// #[tracing::instrument(skip(db))]
// pub fn listen_to_low_sensor(
//     low_sensor_pin: SharedInputPin,
//     pump_control_pin: SharedOutputPin,
//     sensor_state: SharedPinState,
//     delay: u64,
//     db: DbPool,
// ) {
//     let debouncer: Arc<Mutex<Option<SensorDebouncer>>> = Arc::from(Mutex::new(None));
//     let mut pin = low_sensor_pin.lock().unwrap();

//     pin.set_async_interrupt(Trigger::Both, move |level| {
//         let shared_deb = Arc::clone(&debouncer);
//         let mut deb = shared_deb.lock().unwrap();

//         if deb.is_some() {
//             deb.as_mut().unwrap().reset_deadline(level);
//             return;
//         }

//         let debouncer = SensorDebouncer::new(Duration::new(2, 0), level);
//         *deb = Some(debouncer);

//         let sleep = deb.as_ref().unwrap().sleep();
//         let rt = Runtime::new().unwrap();
//         rt.block_on(sleep);
//         *deb = None;
//         drop(deb);

//         rt.block_on(update_low_sensor(
//             level,
//             Arc::clone(&pump_control_pin),
//             Arc::clone(&sensor_state),
//             delay,
//             db.clone(),
//         ));
//     })
//     .expect("Could not not listen on low water level sump pin.");
// }

// #[tracing::instrument()]
// pub fn listen_to_irrigation_low_sensor(
//     irrigation_low_sensor_pin: SharedInputPin,
//     irrigation_pump_control_pin: SharedOutputPin,
//     sensor_state: SharedPinState,
//     delay: u64,
// ) {
//     let debouncer: Arc<Mutex<Option<SensorDebouncer>>> = Arc::from(Mutex::new(None));
//     let mut pin = irrigation_low_sensor_pin.lock().unwrap();

//     pin.set_async_interrupt(Trigger::Both, move |level| {
//         let shared_deb = Arc::clone(&debouncer);
//         let mut deb = shared_deb.lock().unwrap();

//         if deb.is_some() {
//             deb.as_mut().unwrap().reset_deadline(level);
//             return;
//         }

//         let debouncer = SensorDebouncer::new(Duration::new(2, 0), level);
//         *deb = Some(debouncer);

//         let sleep = deb.as_ref().unwrap().sleep();
//         let rt = Runtime::new().unwrap();
//         rt.block_on(sleep);
//         *deb = None;
//         drop(deb);

//         rt.block_on(update_irrigation_low_sensor(
//             level,
//             Arc::clone(&irrigation_pump_control_pin),
//             Arc::clone(&sensor_state),
//             delay,
//         ));
//     })
//     .expect("Could not not listen on low water level sump pin.");
// }

// fn error_trace(e: Error, msg: &str) {
//     tracing::error!(target = module_path!(), error = e.to_string(), msg);
// }

// #[cfg(test)]
// mod tests {
//     use crate::{
//         hydro::{control::Control, gpio::Level},
//         repository::Repo,
//         test_fixtures::gpio::mock_gpio_get,
//     };

//     #[test]
//     fn test_new() {
//         let callback = |level: Level, control: Control, db: Repo, delay: u64| {
//             format!("{:?}", level);
//             ()
//         };
//         let mock_gpio = mock_gpio_get(vec![1]);
//         // TODO: finish
//         // let _sensor: Sensor =
//         //     Sensor::new(1, &mock_gpio, Some(callback), Some(Trigger::Both)).unwrap();
//     }
// }
