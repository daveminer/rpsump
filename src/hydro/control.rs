use crate::hydro::gpio::{Gpio, Level, OutputPin};
use anyhow::{anyhow, Error};
use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

pub type PinLock<'a> = MutexGuard<'a, Box<dyn OutputPin>>;

/// Represents a GPIO output for controlling stateful equipment. Water pump, etc.
pub type SharedOutputPin = Arc<Mutex<Box<dyn OutputPin>>>;

#[derive(Clone)]
pub struct Control {
    pub label: String,
    pub level: Level,
    pub pin: SharedOutputPin,
}

impl Control {
    /// Creates a new output on a GPIO pin.
    pub fn new<G>(label: String, pin: u8, gpio: &G) -> Result<Self, Error>
    where
        G: Gpio,
    {
        let pin = gpio.get(pin)?;
        let pin_io = pin.into_output_low();

        Ok(Self {
            label,
            level: Level::Low,
            pin: Arc::from(Mutex::new(pin_io)),
        })
    }

    fn lock(&self) -> Result<PinLock, Error> {
        self.pin.lock().map_err(|e| anyhow!(e.to_string()))
    }
}

impl fmt::Debug for Control {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Control")
            .field("label", &self.label)
            .field("level", &self.level)
            .finish()
    }
}

pub trait Output {
    fn on(&mut self) -> Result<(), Error>;
    fn off(&mut self) -> Result<(), Error>;
    fn is_on(&self) -> bool;
    fn is_off(&self) -> bool;
}

impl Output for Control {
    // Set the pin high
    fn on(&mut self) -> Result<(), Error> {
        let mut pin = self.lock()?;

        pin.set_high();

        Ok(())
    }

    fn off(&mut self) -> Result<(), Error> {
        let mut pin = self.lock()?;

        pin.set_low();

        Ok(())
    }

    fn is_on(&self) -> bool {
        self.level == Level::High
    }

    fn is_off(&self) -> bool {
        self.level == Level::Low
    }
}

pub fn pin_lock_failure(e: &PoisonError<PinLock>, control: &Control) -> Error {
    tracing::error!(
        target = module_path!(),
        error = e.to_string(),
        "Failed to lock pin {}",
        control.label
    );

    anyhow!(e.to_string())
}

/// Applies a state change to a sensor by settting the pin level, creating a database event,
/// and updating the sensor state.
// #[tracing::instrument(skip(db))]
// pub async fn update_sensor(
//     signal: Level,
//     control: Control,
//     sensor: Sensor,
//     trace_msg: String,
//     db: DbPool,
// ) {
//     // T
//     if signal != sensor.level {

//         if let Err(e) =
//             SumpEvent::create("pump on".to_string(), "reservoir full".to_string(), db).await
//         {
//             tracing::error!(
//                 target = module_path!(),
//                 error = e.to_string(),
//                 "Failed to create sump event for pump on"
//             );
//         }
//     }

//     let mut sensors = sensor_state.lock().unwrap();

//     sensors.high_sensor = level;
// }

/// Applies a state change to the high sensor by settting the pin level, creating a database event,
/// and updating the sensor state.
// #[tracing::instrument(skip(db))]
// pub async fn update_high_sensor(
//     level: Level,
//     pump_control_pin: Arc<Mutex<OutputPin>>,
//     sensor_state: Arc<Mutex<PinState>>,
//     db: DbPool,
// ) {
//     // Turn the pump on
//     if level == Level::High {
//         let mut pin = pump_control_pin.lock().unwrap();
//         pin.set_high();
//         tracing::info!("Sump pump turned on.");

//         if let Err(e) =
//             SumpEvent::create("pump on".to_string(), "reservoir full".to_string(), db).await
//         {
//             tracing::error!(
//                 target = module_path!(),
//                 error = e.to_string(),
//                 "Failed to create sump event for pump on"
//             );
//         }
//     }

//     let mut sensors = sensor_state.lock().unwrap();

//     sensors.high_sensor = level;
// }

/// Applies a state change to the low sensor similar to the high sensor. The difference is that the
/// low sensor accepts a delay that allows the pump to run long to lower the water level enough to
/// prevent signal bouncing.
// #[tracing::instrument(skip(db))]
// pub async fn update_low_sensor(
//     level: Level,
//     pump_control_pin: Arc<Mutex<OutputPin>>,
//     sensor_state: Arc<Mutex<PinState>>,
//     delay: u64,
//     db: DbPool,
// ) {
//     // Turn the pump off
//     if level == Level::Low {
//         if delay > 0 {
//             thread::sleep(Duration::from_millis(delay as u64 * 1000));
//         }

//         let mut pin = pump_control_pin.lock().unwrap();
//         pin.set_low();
//         tracing::info!(target = module_path!(), "Sump pump turned off");

//         if let Err(e) =
//             SumpEvent::create("pump off".to_string(), "reservoir empty".to_string(), db).await
//         {
//             tracing::error!(
//                 target = module_path!(),
//                 error = e.to_string(),
//                 "Failed to create sump event for pump off"
//             );
//         }
//     }

//     let mut sensors = sensor_state.lock().unwrap();

//     sensors.low_sensor = level;
// }

// #[tracing::instrument()]
// pub async fn update_irrigation_low_sensor(
//     level: Level,
//     irrigation_pump_control_pin: Arc<Mutex<OutputPin>>,
//     sensor_state: Arc<Mutex<PinState>>,
//     delay: u64,
// ) {
//     tracing::info!(
//         target = module_path!(),
//         level = level.to_string(),
//         "Changing irrigation low sensor"
//     );

//     let mut sensors = sensor_state.lock().unwrap();
//     sensors.irrigation_low_sensor = level;
// }

#[cfg(test)]
mod tests {
    use crate::test_fixtures::gpio::mock_gpio_get;

    use super::Control;

    #[test]
    fn test_format() {
        let mock_gpio = mock_gpio_get(1);

        let control: Control =
            Control::new("control pin label".to_string(), 1, &mock_gpio).unwrap();

        assert_eq!(
            format!("{:?}", control),
            "Control { label: \"control pin label\", level: Low }"
        );
    }

    #[test]
    fn test_lock() {
        let mock_gpio = mock_gpio_get(1);

        let control: Control =
            Control::new("control pin label".to_string(), 1, &mock_gpio).unwrap();

        let _lock = control.lock().unwrap();
    }
}
