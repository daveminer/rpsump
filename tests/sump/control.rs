// TODO: needs Raspberry Pi environment
// #[cfg(test)]
// mod tests {
//     use rppal::gpio::{Gpio, Level, OutputPin};
//     use std::sync::{Arc, Mutex};

//     use crate::common::test_app::spawn_test_db;
//     use rpsump::database::new_pool;
//     use rpsump::sump::{control, sensor::PinState};

//     // Helper function to create a GPIO output pin for testing
//     fn create_output_pin() -> OutputPin {
//         let gpio = Gpio::new();
//         let mut pin = gpio.get(1).unwrap().into_output();
//         pin.set_high();

//         pin
//     }

//     #[tokio::test]
//     async fn test_update_high_sensor() {
//         // Create a mock database pool
//         let db_pool = new_pool(&spawn_test_db());

//         // Create a mock GPIO output pin
//         let pump_control_pin = Arc::new(Mutex::new(create_output_pin()));

//         // Create a mock sensor state
//         let sensor_state = Arc::new(Mutex::new(PinState {
//             high_sensor: Level::High,
//             low_sensor: Level::Low,
//         }));

//         // Call the update_high_sensor function with a level of High
//         control::update_high_sensor(
//             Level::Low,
//             Arc::clone(&pump_control_pin),
//             Arc::clone(&sensor_state),
//             db_pool,
//         )
//         .await;

//         // Verify that the pump control pin is set to High
//         assert!(pump_control_pin.lock().unwrap().is_set_low());

//         // Verify that the high_sensor state is set to High
//         assert_eq!(sensor_state.lock().unwrap().high_sensor, Level::Low);
//     }

//     #[tokio::test]
//     async fn test_update_low_sensor() {
//         // Create a mock database pool
//         // Create a mock database pool
//         let db_pool = new_pool(&spawn_test_db());

//         // Create a mock GPIO output pin
//         let pump_control_pin = Arc::new(Mutex::new(create_output_pin()));

//         // Create a mock sensor state
//         let sensor_state = Arc::new(Mutex::new(PinState {
//             high_sensor: Level::Low,
//             low_sensor: Level::Low,
//         }));

//         // Call the update_low_sensor function with a level of Low and a delay of 0
//         control::update_low_sensor(
//             Level::Low,
//             pump_control_pin.clone(),
//             sensor_state.clone(),
//             0,
//             db_pool,
//         )
//         .await;

//         // Verify that the pump control pin is set to Low
//         assert!(pump_control_pin.lock().unwrap().is_set_high());

//         // Verify that the low_sensor state is set to Low
//         assert_eq!(sensor_state.lock().unwrap().low_sensor, Level::High);
//     }
// }
