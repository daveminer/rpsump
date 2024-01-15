pub mod control;
pub mod debounce;
pub mod gpio;
mod irrigator;
mod pump;
pub mod schedule;
pub mod sensor;
mod sump;

use self::control::Control;
use self::irrigator::Irrigator;
use self::sensor::Sensor;
use self::sump::Sump;
use crate::config::HydroConfig;
use crate::database::DbPool;

use anyhow::Error;
use gpio::{Gpio, Level};

#[derive(Clone)]
pub struct Hydro {
    pub db_pool: DbPool,
    pub sump: Sump,
    pub irrigator: Irrigator,
}

impl Hydro {
    pub fn new<C, G>(
        db: &DbPool,
        config: &HydroConfig,
        gpio: &G,
        high_sensor_handler: C,
        low_sensor_handler: C,
        irrigator_empty_sensor_handler: C,
    ) -> Result<Self, Error>
    where
        C: FnMut(Level) + Send + 'static,
        G: Gpio,
    {
        let sump = Sump::new(&config.sump, gpio, high_sensor_handler, low_sensor_handler)?;
        let irrigator = Irrigator::new(&config.irrigation, gpio, irrigator_empty_sensor_handler)?;

        Ok(Self {
            db_pool: db.clone(),
            irrigator,
            sump,
        })
    }
}
// impl Sumpp {
//     // Creates a new sump struct with sensors and their state.
//     pub fn new(db_pool: DbPool, config: &SumpConfig) -> Result<Self, Error> {
//         // create the GPIO pins
//         let gpio = Gpio::new()?;

//         let high_sensor_pin_io = gpio.get(config.high_sensor_pin)?.into_input_pullup();
//         let high_sensor_reading = high_sensor_pin_io.read();
//         let high_sensor_pin = Arc::from(Mutex::new(high_sensor_pin_io));
//         let high_debounce = Arc::from(Mutex::new(None));

//         let low_sensor_pin_io = gpio.get(config.low_sensor_pin)?.into_input_pullup();
//         let low_sensor_reading = low_sensor_pin_io.read();
//         let low_sensor_pin = Arc::from(Mutex::new(low_sensor_pin_io));
//         let low_debounce = Arc::from(Mutex::new(None));

//         let irrigation_low_sensor_pin_io = gpio.get(config.low_sensor_pin)?.into_input_pullup();
//         let irrigation_low_sensor_reading = irrigation_low_sensor_pin_io.read();
//         let irrigation_low_sensor_pin = Arc::from(Mutex::new(irrigation_low_sensor_pin_io));
//         let irrigation_low_debounce = Arc::from(Mutex::new(None));

//         let pump_control_pin: Arc<Mutex<OutputPin>> = Arc::from(Mutex::new(
//             gpio.get(config.pump_control_pin)?.into_output_low(),
//         ));

//         let irrigation_pump_control_pin: Arc<Mutex<OutputPin>> = Arc::from(Mutex::new(
//             gpio.get(config.irrigation.pump_control_pin)?
//                 .into_output_low(),
//         ));

//         let irrigation_valve_1_control_pin: Arc<Mutex<OutputPin>> = Arc::from(Mutex::new(
//             gpio.get(config.irrigation.valve_1_control_pin)?
//                 .into_output_low(),
//         ));
//         let irrigation_valve_2_control_pin: Arc<Mutex<OutputPin>> = Arc::from(Mutex::new(
//             gpio.get(config.irrigation.valve_2_control_pin)?
//                 .into_output_low(),
//         ));
//         let irrigation_valve_3_control_pin: Arc<Mutex<OutputPin>> = Arc::from(Mutex::new(
//             gpio.get(config.irrigation.valve_3_control_pin)?
//                 .into_output_low(),
//         ));
//         let irrigation_valve_4_control_pin: Arc<Mutex<OutputPin>> = Arc::from(Mutex::new(
//             gpio.get(config.irrigation.valve_4_control_pin)?
//                 .into_output_low(),
//         ));

//         // Read initial state of inputs
//         let sensor_state = Arc::from(Mutex::new(PinState {
//             high_sensor: high_sensor_reading,
//             low_sensor: low_sensor_reading,
//             irrigation_low_sensor: irrigation_low_sensor_reading,
//         }));

//         listen_to_high_sensor(
//             Arc::clone(&high_sensor_pin),
//             Arc::clone(&pump_control_pin),
//             Arc::clone(&sensor_state),
//             db_pool.clone(),
//         );

//         listen_to_low_sensor(
//             Arc::clone(&low_sensor_pin),
//             Arc::clone(&pump_control_pin),
//             Arc::clone(&sensor_state),
//             config.pump_shutoff_delay,
//             db_pool.clone(),
//         );

//         listen_to_irrigation_low_sensor(
//             Arc::clone(&low_sensor_pin),
//             Arc::clone(&pump_control_pin),
//             Arc::clone(&sensor_state),
//             config.pump_shutoff_delay,
//         );

//         Ok(Sump {
//             db_pool,
//             high_sensor_debounce: Arc::clone(&high_debounce),
//             high_sensor_pin: Arc::clone(&high_sensor_pin),
//             low_sensor_debounce: Arc::clone(&low_debounce),
//             low_sensor_pin: Arc::clone(&low_sensor_pin),
//             irrigation_enabled: config.irrigation.enabled,
//             irrigation_low_sensor_debounce: Arc::clone(&irrigation_low_debounce),
//             irrigation_low_sensor_pin: Arc::clone(&irrigation_low_sensor_pin),
//             irrigation_pump_control_pin: Arc::clone(&irrigation_pump_control_pin),
//             irrigation_valve_1_control_pin,
//             irrigation_valve_2_control_pin,
//             irrigation_valve_3_control_pin,
//             irrigation_valve_4_control_pin,
//             pump_control_pin: Arc::clone(&pump_control_pin),
//             sensor_state: Arc::clone(&sensor_state),
//         })
//     }
// }

// TODO: REPLACE

// #[tracing::instrument]
// pub fn spawn_reporting_thread(
//     sensor_state: SharedPinState,
//     interval_seconds: u64,
// ) -> thread::JoinHandle<()> {
//     thread::spawn(move || {
//         let mut start_time = Instant::now();
//         let sensors = Arc::clone(&sensor_state);

//         loop {
//             // Report to console
//             let _sensor_reading = *sensors.lock().unwrap();

//             // Wait for N seconds
//             let elapsed_time = start_time.elapsed();
//             if elapsed_time < Duration::from_secs(interval_seconds) {
//                 thread::sleep(Duration::from_secs(interval_seconds) - elapsed_time);
//             }
//             start_time = Instant::now();
//         }
//     })
// }
