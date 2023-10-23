use std::time::{Duration as StdDuration, SystemTime};

use anyhow::{anyhow, Error};
use futures::executor::block_on;
use rppal::gpio::Level;
use tokio::task::JoinHandle;
use tokio::time::sleep;

use crate::database::DbPool;
use crate::sump::schedule::IrrigationEvent;
use crate::sump::{SharedOutputPin, Sump};

static INVALID_HOSE_NUMBER_MSG: &str = "Invalid hose number provided";

pub fn run_next_event(db: DbPool, sump: Sump) {
    // Get the next event
    let (duration, event) = match IrrigationEvent::next_queued(db.clone()) {
        Ok(event) => event,
        Err(e) => {
            tracing::error!("Error getting next irrigation event: {:?}", e);
            return;
        }
    };

    // Start the irrigation
    let _irrigation_job = start_irrigation(db, event, duration, sump);
}

fn start_irrigation(
    db: DbPool,
    event: IrrigationEvent,
    duration: i32,
    sump: Sump,
) -> JoinHandle<Result<(), Error>> {
    let sump = sump.clone();
    let event = event.clone();
    tokio::spawn(async move {
        let sensor_state = match sump.sensor_state.lock() {
            Ok(sensor_state) => *sensor_state,
            Err(e) => {
                tracing::error!("Could not get sensor state: {}", e);
                // TODO: send email
                return Err(anyhow!(e.to_string()));
            }
        };
        if sensor_state.irrigation_low_sensor == Level::Low {
            // Exit if water is too low
            tracing::warn!("Water is too low to start irrigation.");
            ()
        }

        let hose_pin = match choose_irrigation_valve_pin(event.hose_id, sump.clone()) {
            Ok(pin) => pin,
            Err(e) => {
                tracing::error!("Invalid pin from schedule");
                return Err(e);
            }
        };

        // Open the solenoid for the job
        let mut hose = hose_pin.lock().unwrap();
        // Start the pump
        hose.set_high();
        drop(hose);

        let start_time = SystemTime::now();
        // Start the pump
        let mut pump = match sump.irrigation_pump_control_pin.lock() {
            Ok(pump) => pump,
            Err(e) => {
                tracing::error!("Could not get sump pump control pin: {}", e);
                return Err(anyhow!(e.to_string()));
            }
        };

        pump.set_high();
        drop(pump);

        // Wait for the job to finish
        if duration > 60 {
            tracing::error!("Schedule duration is too long");
            return Err(anyhow!("Schedule duration is too long"));
        }

        while !job_complete(duration, start_time) {
            block_on(sleep(StdDuration::from_secs(1)));
        }

        // Stop the pump
        let mut pump = match sump.irrigation_pump_control_pin.lock() {
            Ok(pump) => pump,
            Err(e) => {
                tracing::error!("Could not get sump pump control pin: {}", e);
                return Err(anyhow!(e.to_string()));
            }
        };
        pump.set_low();

        // Close the solenoid
        let mut hose = hose_pin.lock().unwrap();
        // Stop the pump
        hose.set_low();

        // Move the job out of "in progress" status
        let _ = block_on(IrrigationEvent::finish(db));
        return Ok(());
    })
}

fn choose_irrigation_valve_pin(hose_id: i32, sump: Sump) -> Result<SharedOutputPin, Error> {
    if hose_id == 1 {
        Ok(sump.irrigation_valve_1_control_pin)
    } else if hose_id == 2 {
        Ok(sump.irrigation_valve_2_control_pin)
    } else if hose_id == 3 {
        Ok(sump.irrigation_valve_3_control_pin)
    } else if hose_id == 4 {
        Ok(sump.irrigation_valve_4_control_pin)
    } else {
        tracing::error!(INVALID_HOSE_NUMBER_MSG);
        Err(anyhow!(INVALID_HOSE_NUMBER_MSG))
    }
}

fn job_complete(duration: i32, start_time: SystemTime) -> bool {
    let elapsed = match SystemTime::now().duration_since(start_time) {
        Ok(now) => now,
        Err(e) => {
            tracing::error!("Error getting duration since start time: {:?}", e);
            return true;
        }
    };

    let dur = match duration.try_into() {
        Ok(dur) => dur,
        Err(e) => {
            tracing::error!("Error converting duration to std duration: {:?}", e);
            return true;
        }
    };

    elapsed >= StdDuration::from_secs(dur)
}
