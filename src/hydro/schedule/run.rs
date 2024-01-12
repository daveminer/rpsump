use std::time::{Duration as StdDuration, SystemTime};

use anyhow::{anyhow, Error};
use futures::executor::block_on;
use rppal::gpio::Level;
use tokio::task::JoinHandle;
use tokio::time::sleep;

use crate::database::DbPool;
use crate::hydro::schedule::IrrigationEvent;
use crate::hydro::Sump;

static INVALID_HOSE_NUMBER_MSG: &str = "Invalid hose number provided";

// #[tracing::instrument(skip(db))]
// pub async fn run_next_event(db: DbPool, sump: Sump) {
//     // Get the next event
//     let (duration, event) = match IrrigationEvent::next_queued(db.clone()).await {
//         Ok(event) => event,
//         Err(e) => {
//             tracing::error!(
//                 target = module_path!(),
//                 error = e.to_string(),
//                 "Error getting next irrigation event"
//             );
//             return;
//         }
//     };

//     // Start the irrigation
//     let _irrigation_job = start_irrigation(db, event, duration, sump);
// }

// #[tracing::instrument(skip(db))]
// fn start_irrigation(
//     db: DbPool,
//     event: IrrigationEvent,
//     duration: i32,
//     sump: Sump,
// ) -> JoinHandle<Result<(), Error>> {
//     let sump = sump.clone();
//     let event = event.clone();
//     tokio::spawn(async move {
//         let sensor_state = match sump.sensor_state.lock() {
//             Ok(sensor_state) => *sensor_state,
//             Err(e) => {
//                 tracing::error!(
//                     target = module_path!(),
//                     error = e.to_string(),
//                     "Could not get sensor state"
//                 );
//                 return Err(anyhow!(e.to_string()));
//             }
//         };
//         if sensor_state.irrigation_low_sensor == Level::Low {
//             // Exit if water is too low
//             tracing::warn!(
//                 target = module_path!(),
//                 "Water is too low to start irrigation."
//             );
//             return Ok(());
//         }

//         let hose_pin = match choose_irrigation_valve_pin(event.hose_id, sump.clone()) {
//             Ok(pin) => pin,
//             Err(e) => {
//                 tracing::error!(
//                     target = module_path!(),
//                     error = e.to_string(),
//                     hose_id = event.hose_id,
//                     "Invalid pin from schedule"
//                 );
//                 return Err(e);
//             }
//         };

//         tracing::info!(target = module_path!(), "Starting irrigation job");
//         // Open the solenoid for the job
//         let mut hose = hose_pin.lock().unwrap();
//         // Start the pump
//         hose.set_high();
//         drop(hose);

//         let start_time = SystemTime::now();
//         // Start the pump
//         let mut pump = match sump.irrigation_pump_control_pin.lock() {
//             Ok(pump) => pump,
//             Err(e) => {
//                 tracing::error!(
//                     target = module_path!(),
//                     error = e.to_string(),
//                     "Could not get irrigation pump control pin on start"
//                 );
//                 return Err(anyhow!(e.to_string()));
//             }
//         };

//         pump.set_high();
//         drop(pump);

//         // Wait for the job to finish
//         while !job_complete(duration, start_time) {
//             block_on(sleep(StdDuration::from_secs(1)));
//         }

//         // Stop the pump
//         let mut pump = match sump.irrigation_pump_control_pin.lock() {
//             Ok(pump) => pump,
//             Err(e) => {
//                 tracing::error!(
//                     target = module_path!(),
//                     error = e.to_string(),
//                     "Could not get irrigation pump control pin on stop"
//                 );
//                 return Err(anyhow!(e.to_string()));
//             }
//         };

//         tracing::error!(target = module_path!(), "Stopping irrigation job");
//         pump.set_low();

//         // Close the solenoid
//         let mut hose = hose_pin.lock().unwrap();
//         // Stop the pump
//         hose.set_low();

//         // Move the job out of "in progress" status
//         let _ = block_on(IrrigationEvent::finish(db));
//         return Ok(());
//     })
// }

// fn choose_irrigation_valve_pin(hose_id: i32, sump: Sump) -> Result<SharedOutputPin, Error> {
//     if hose_id == 1 {
//         Ok(sump.irrigation_valve_1_control_pin)
//     } else if hose_id == 2 {
//         Ok(sump.irrigation_valve_2_control_pin)
//     } else if hose_id == 3 {
//         Ok(sump.irrigation_valve_3_control_pin)
//     } else if hose_id == 4 {
//         Ok(sump.irrigation_valve_4_control_pin)
//     } else {
//         Err(anyhow!(INVALID_HOSE_NUMBER_MSG))
//     }
// }

fn job_complete(duration: i32, start_time: SystemTime) -> bool {
    let elapsed = match SystemTime::now().duration_since(start_time) {
        Ok(now) => now,
        Err(e) => {
            tracing::error!(
                target = module_path!(),
                error = e.to_string(),
                "Error getting duration since start time"
            );
            return true;
        }
    };

    let dur = match duration.try_into() {
        Ok(dur) => dur,
        Err(_e) => {
            tracing::error!(
                target = module_path!(),
                "Error converting duration to std duration"
            );
            return true;
        }
    };

    elapsed >= StdDuration::from_secs(dur)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, SystemTime};

    #[test]
    fn test_job_complete() {
        // Set up test data
        let duration = 60;
        let start_time = SystemTime::now() - Duration::from_secs(30);

        // Call the function being tested
        let result = job_complete(duration, start_time);

        // Check the result
        assert_eq!(result, false);
    }
}
