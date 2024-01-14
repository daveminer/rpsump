use anyhow::{anyhow, Error};
use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};

use crate::hydro::{
    debounce::Debouncer,
    gpio::{Gpio, InputPin, Trigger},
    Level,
};

/// Sensors will trigger async callbacks (which create a thread) on these
pub type SharedInputPin = Arc<Mutex<Box<dyn InputPin>>>;
/// Threads spawned from sensor state changes will share one of these per sensor
pub type SharedSensorDebouncer = Arc<Mutex<Option<Debouncer>>>;

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
impl Sensor {
    pub fn new<C, G>(
        pin_number: u8,
        gpio: &G,
        handler: Option<C>,
        trigger: Option<Trigger>,
    ) -> Result<Self, Error>
    where
        C: FnMut(Level) + Send + 'static,
        G: Gpio,
    {
        let mut pin_io = gpio
            .get(pin_number)
            .map_err(|e| anyhow!(e))?
            .into_input_pullup();

        if handler.is_some() {
            if trigger.is_none() {
                return Err(anyhow!("Cannot set interrupt handler without trigger"));
            }

            // TODO: fix this stuff
            let _debouncer: Arc<Mutex<Option<Debouncer>>> = Arc::from(Mutex::new(None));

            // TODO: remove unwrap
            let _ = pin_io.set_async_interrupt(trigger.unwrap(), Box::new(handler.unwrap()));
        }

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

// fn serialize_level<S>(level: &Level, serializer: S) -> Result<S::Ok, S::Error>
// where
//     S: Serializer,
// {
//     serializer.serialize_u8(*level as u8)
// }

// fn deserialize_level<'de, D>(deserializer: D) -> Result<Level, D::Error>
// where
//     D: Deserializer<'de>,
// {
//     let value = u8::deserialize(deserializer)?;
//     match value {
//         0 => Ok(Level::Low),
//         1 => Ok(Level::High),
//         _ => Err(serde::de::Error::custom("invalid Level value")),
//     }
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
// pub fn listen_to_high_sensor(
//     high_sensor_pin: SharedInputPin,
//     pump_control_pin: SharedOutputPin,
//     sensor_state: SharedPinState,
//     db: DbPool,
// ) {
//     let debouncer: Arc<Mutex<Option<SensorDebouncer>>> = Arc::from(Mutex::new(None));
//     let mut pin = high_sensor_pin.lock().unwrap();

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

//         rt.block_on(update_high_sensor(
//             level,
//             Arc::clone(&pump_control_pin),
//             Arc::clone(&sensor_state),
//             db.clone(),
//         ));
//     })
//     .expect("Could not not listen on high water level sump pin.");
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

#[cfg(test)]
mod tests {
    use crate::{
        hydro::gpio::{Level, Trigger},
        test_fixtures::gpio::mock_gpio_get,
    };

    use super::Sensor;

    #[test]
    fn test_new() {
        let callback = |level: Level| {
            format!("{:?}", level);
            ()
        };
        let mock_gpio = mock_gpio_get(1);
        let _sensor: Sensor =
            Sensor::new(1, &mock_gpio, Some(callback), Some(Trigger::Both)).unwrap();
    }
}
