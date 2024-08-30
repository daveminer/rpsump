use anyhow::{anyhow, Error};
use std::time::{Duration, SystemTime};

use tokio::time::sleep;

use crate::hydro::{control::Control, schedule::IrrigationEvent, sensor::Input, Irrigator};
use crate::repository::Repo;

pub async fn run_irrigation_event(repo: Repo, irrigator: &Irrigator) {
    // Get the next event
    let (event, schedule) = match repo.next_queued_irrigation_event().await {
        Ok(dur_event) => match dur_event {
            Some(dur_event) => dur_event,
            None => return,
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
    if let Err(err) = irrigate(repo, event, schedule.duration, irrigator).await {
        tracing::error!(
            target = module_path!(),
            error = err.to_string(),
            "Failed to start irrigation"
        );
    }
}

#[tracing::instrument(skip(irrigator, repo))]
pub async fn irrigate(
    repo: Repo,
    event: IrrigationEvent,
    duration: i32,
    irrigator: &Irrigator,
) -> Result<(), Error> {
    tracing::info!(target = module_path!(), "Starting irrigation job");
    let start_time = SystemTime::now();

    match repo.begin_irrigation(event.clone()).await {
        Ok(()) => (),
        Err(e) => {
            tracing::error!(
                target = module_path!(),
                error = e.to_string(),
                "Error beginning irrigation event"
            );
            return Err(anyhow!(e.to_string()));
        }
    }

    let hose = match event_hose_pin(&event, irrigator) {
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

    // Open the solenoid and start the pump
    let mut hose_lock = hose_pin.lock().await;
    hose_lock.on();
    drop(hose_lock);

    let mut pump_lock = pump_pin.lock().await;
    pump_lock.on();
    drop(pump_lock);

    // // Wait for the job to finish
    let duration = Duration::from_secs(duration as u64);
    let mut is_job_done = job_complete(duration, start_time);
    while !is_job_done {
        sleep(tokio::time::Duration::from_secs(1)).await;
        is_job_done = job_complete(duration, start_time);
    }

    tracing::info!(target = module_path!(), "Stopping irrigation job");

    // Stop the pump and close the solenoid
    let mut pump_lock = irrigator.pump.pin.lock().await;
    pump_lock.off();

    // Open the solenoid and start the pump
    let mut hose_lock = hose.pin.lock().await;
    hose_lock.off();

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
}

fn event_hose_pin(event: &IrrigationEvent, irrigator: &Irrigator) -> Result<Control, Error> {
    let hose_id = event.hose_id;
    if hose_id == 1 {
        Ok(irrigator.valve1.clone())
    } else if hose_id == 2 {
        Ok(irrigator.valve2.clone())
    } else if hose_id == 3 {
        Ok(irrigator.valve3.clone())
    } else if hose_id == 4 {
        Ok(irrigator.valve4.clone())
    } else {
        tracing::error!(
            target = module_path!(),
            error = "Invalid pin from schedule",
            hose_id = event.hose_id,
            "Invalid pin from schedule"
        );
        Err(anyhow!("Invalid hose number provided"))
    }
}

fn job_complete(duration: Duration, start_time: SystemTime) -> bool {
    match SystemTime::now().duration_since(start_time) {
        Ok(elapsed) => elapsed >= duration,
        Err(e) => {
            tracing::error!(
                target = module_path!(),
                error = e.to_string(),
                "Could not get duration since start time for job complete calculation"
            );
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate;
    use rstest::rstest;

    use crate::hydro::control::Control;
    use crate::hydro::gpio::{Level, MockGpio, MockInputPin, MockPin, Trigger};
    use crate::hydro::irrigator::Irrigator;
    use crate::hydro::schedule::run::{event_hose_pin, job_complete, run_irrigation_event};
    use crate::hydro::sensor::Sensor;
    use crate::hydro::signal::Message;
    use crate::repository::models::irrigation_event::IrrigationEvent;
    use crate::test_fixtures::gpio::mock_gpio_get;
    use crate::{
        repository::{MockRepository, Repository},
        test_fixtures::irrigation::{event::completed_event, irrigator::irrigator},
    };

    use std::time::{Duration, SystemTime};

    #[rstest]
    #[tokio::test]
    async fn test_run_next_event() {
        let mut mock_repo = MockRepository::new();

        let _ = mock_repo
            .expect_next_queued_irrigation_event()
            .returning(|| Ok(None));

        let _ = mock_repo
            .expect_finish_irrigation_event()
            .returning(|| Ok(()));
        let repo = Box::new(mock_repo);

        let repo_static: &'static dyn Repository = Box::leak(repo);

        // TODO: combine this with the irrigation fixture in a way that
        // provides the runtime for unit tests (rt cannot drop in sync context)
        let mut mock_gpio: MockGpio = mock_gpio_get(vec![1, 2, 3, 4, 5]);
        let (tx, _rx) = tokio::sync::mpsc::channel(32);

        mock_gpio
            .expect_get()
            .with(predicate::eq(6))
            .times(1)
            .returning(|_| {
                let mut pin = MockPin::new();
                pin.expect_into_input_pullup().times(1).returning(|| {
                    let mut input_pin = MockInputPin::new();
                    input_pin
                        .expect_set_async_interrupt()
                        .times(1)
                        .returning(|_, _, _, _, _| Ok(()));
                    input_pin.expect_read().times(1).returning(|| Level::High);
                    Box::new(input_pin)
                });
                Ok(Box::new(pin))
            });

        let pump = Control::new("Pump".to_string(), 1, &mock_gpio).unwrap();
        let valve1 = Control::new("Valve1".to_string(), 2, &mock_gpio).unwrap();
        let valve2 = Control::new("Valve2".to_string(), 3, &mock_gpio).unwrap();
        let valve3 = Control::new("Valve3".to_string(), 4, &mock_gpio).unwrap();
        let valve4 = Control::new("Valve4".to_string(), 5, &mock_gpio).unwrap();

        let handle = tokio::runtime::Handle::current();

        let low_sensor = Sensor::new(
            Message::SumpEmpty,
            6,
            &mock_gpio,
            Trigger::Both,
            &tx,
            handle.clone(),
        )
        .unwrap();

        let irrigator = Irrigator {
            low_sensor,
            pump,
            valve1,
            valve2,
            valve3,
            valve4,
        };

        run_irrigation_event(repo_static, &irrigator).await;
    }

    #[rstest]
    fn test_event_hose_pin(completed_event: IrrigationEvent, irrigator: Irrigator) {
        let result = event_hose_pin(&completed_event, &irrigator).unwrap();
        assert_eq!(result, irrigator.valve1);
    }

    #[test]
    fn test_job_complete() {
        // Set up test data
        let duration = Duration::from_secs(60);
        let earlier_start_time = SystemTime::now() - Duration::from_secs(90);
        let later_start_time = SystemTime::now() - Duration::from_secs(30);

        // Call the function being tested
        let shorter_result = job_complete(duration, later_start_time);
        let longer_result = job_complete(duration, earlier_start_time);
        assert_eq!(shorter_result, false);
        assert_eq!(longer_result, true);
    }
}
