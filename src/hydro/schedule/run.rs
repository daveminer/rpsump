use std::time::{Duration as StdDuration, SystemTime};

use anyhow::{anyhow, Error};
use tokio::{sync::MutexGuard, task::JoinHandle, time::sleep};

use crate::database::RealDbPool;
use crate::hydro::gpio::OutputPin;
use crate::hydro::{control::Control, schedule::IrrigationEvent, sensor::Input, Irrigator};

#[tracing::instrument(skip(db))]
pub async fn run_next_event(db: RealDbPool, irrigator: Irrigator) {
    // Get the next event
    let (duration, event) = match IrrigationEvent::next_queued(db.clone()).await {
        Ok(event) => event,
        Err(e) => {
            tracing::error!(
                target = module_path!(),
                error = e.to_string(),
                "Error getting next irrigation event"
            );
            return;
        }
    };

    if irrigator.low_sensor.is_low() {
        tracing::warn!(
            target = module_path!(),
            "Water level is too low to start irrigation."
        );
        return;
    }

    // Start the irrigation
    let _irrigation_job = start_irrigation(db, event, duration, irrigator);
}

#[tracing::instrument(skip(db))]
async fn start_irrigation(
    db: RealDbPool,
    event: IrrigationEvent,
    duration: i32,
    irrigator: Irrigator,
) -> Result<JoinHandle<Result<(), anyhow::Error>>, Error> {
    let irrigator = irrigator.clone();
    let handle = tokio::spawn(async move {
        tracing::info!(target = module_path!(), "Starting irrigation job");
        let start_time = SystemTime::now();

        let hose_pin = match event_hose_pin(&event, &irrigator) {
            Ok(pin) => pin,
            Err(e) => {
                tracing::error!(
                    target = module_path!(),
                    error = e.to_string(),
                    "Invalid pin from schedule"
                );
                return Err(anyhow!(e.to_string()));
            }
        };

        let (mut hose_lock, mut pump_lock) = match lock_pins(&hose_pin, &irrigator).await {
            Ok((hose_lock, pump_lock)) => (hose_lock, pump_lock),
            Err(e) => {
                tracing::error!(
                    target = module_path!(),
                    error = e.to_string(),
                    "Could not lock irrigation pins"
                );
                return Err(anyhow!(e.to_string()));
            }
        };

        hose_lock.set_high();
        pump_lock.set_high();
        drop(hose_lock);
        drop(pump_lock);

        // Wait for the job to finish
        while !job_complete(duration, start_time) {
            sleep(tokio::time::Duration::from_secs(1)).await;
        }

        // Re-lock the pins
        let (mut hose_lock, mut pump_lock) = match lock_pins(&hose_pin, &irrigator).await {
            Ok((hose_lock, pump_lock)) => (hose_lock, pump_lock),
            Err(e) => {
                tracing::error!(
                    target = module_path!(),
                    error = e.to_string(),
                    "Could not lock irrigation pins to finish job; still running."
                );
                return Err(anyhow!(e.to_string()));
            }
        };

        tracing::error!(target = module_path!(), "Stopping irrigation job");

        // Stop the pump and close the solenoid
        hose_lock.set_low();
        pump_lock.set_low();

        // Move the job out of "in progress" status
        if let Err(e) = IrrigationEvent::finish(db).await {
            tracing::error!(
                target = module_path!(),
                error = e.to_string(),
                "Error finishing irrigation job"
            );
            return Err(anyhow!(e.to_string()));
        }

        drop(hose_lock);
        drop(pump_lock);

        Ok(())
    });

    Ok(handle)
}

fn choose_irrigation_valve_pin(hose_id: i32, irrigator: &Irrigator) -> Result<Control, Error> {
    if hose_id == 1 {
        Ok(irrigator.valve1.clone())
    } else if hose_id == 2 {
        Ok(irrigator.valve2.clone())
    } else if hose_id == 3 {
        Ok(irrigator.valve3.clone())
    } else if hose_id == 4 {
        Ok(irrigator.valve4.clone())
    } else {
        Err(anyhow!("Invalid hose number provided"))
    }
}

fn event_hose_pin(event: &IrrigationEvent, irrigator: &Irrigator) -> Result<Control, Error> {
    match choose_irrigation_valve_pin(event.hose_id, irrigator) {
        Ok(pin) => Ok(pin),
        Err(e) => {
            tracing::error!(
                target = module_path!(),
                error = e.to_string(),
                hose_id = event.hose_id,
                "Invalid pin from schedule"
            );
            return Err(e);
        }
    }
}

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

async fn lock_pins<'a>(
    hose_pin: &'a Control,
    irrigator: &'a Irrigator,
) -> Result<
    (
        MutexGuard<'a, Box<(dyn OutputPin)>>,
        MutexGuard<'a, Box<(dyn OutputPin)>>,
    ),
    anyhow::Error,
> {
    // Lock the solenoid
    let hose = hose_pin.lock().await;

    // Lock the pump
    let pump = irrigator.pump.lock().await;

    Ok((hose, pump))
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
