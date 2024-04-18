use anyhow::{anyhow, Error};
use std::time::{Duration as StdDuration, SystemTime};

use tokio::{task::JoinHandle, time::sleep};

use crate::hydro::{control::Control, schedule::IrrigationEvent, sensor::Input, Irrigator};
use crate::repository::Repo;

#[tracing::instrument(skip(repo))]
pub async fn run_next_event(repo: Repo, irrigator: Irrigator) {
    // Get the next event
    let (duration, event) = match repo.next_queued_irrigation_event().await {
        Ok(dur_event) => match dur_event {
            Some(dur_event) => dur_event,
            None => {
                return;
            }
        },
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
    let _irrigation_job = start_irrigation(repo, event, duration, irrigator);
}

#[tracing::instrument(skip(repo))]
async fn start_irrigation(
    repo: Repo,
    event: IrrigationEvent,
    duration: i32,
    irrigator: Irrigator,
) -> Result<JoinHandle<Result<(), anyhow::Error>>, Error> {
    //let irrigator = irrigator.clone();
    let handle = tokio::spawn(async move {
        tracing::info!(target = module_path!(), "Starting irrigation job");
        let start_time = SystemTime::now();

        let hose = match event_hose_pin(&event, &irrigator) {
            Ok(hose) => hose,
            Err(e) => {
                tracing::error!(
                    target = module_path!(),
                    error = e.to_string(),
                    "Invalid pin from schedule"
                );
                return Err(anyhow!(e.to_string()));
            }
        };

        let hose_pin = hose.pin.clone();
        let pump_pin = irrigator.pump.pin.clone();

        let _ = tokio::task::spawn_blocking(|| async move {
            // Open the solenoid and start the pump
            let mut hose_lock = hose_pin.lock().await;
            hose_lock.on();

            let mut pump_lock = pump_pin.lock().await;
            pump_lock.on();
        })
        .await;

        // Wait for the job to finish
        while !job_complete(duration, start_time) {
            sleep(tokio::time::Duration::from_secs(1)).await;
        }

        tracing::error!(target = module_path!(), "Stopping irrigation job");

        // Stop the pump and close the solenoid
        let hose_pin = hose.pin.clone();
        let pump_pin = irrigator.pump.pin.clone();

        let _ = tokio::task::spawn_blocking(|| async move {
            let mut pump_lock = pump_pin.lock().await;
            pump_lock.off();

            // Open the solenoid and start the pump
            let mut hose_lock = hose_pin.lock().await;
            hose_lock.off();
        })
        .await;

        // Move the job out of "in progress" status
        if let Err(e) = repo.finish_irrigation_event().await {
            tracing::error!(
                target = module_path!(),
                error = e.to_string(),
                "Error finishing irrigation job"
            );
            return Err(anyhow!(e.to_string()));
        }

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
            Err(e)
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
