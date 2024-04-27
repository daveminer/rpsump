pub mod check;
pub mod run;

use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

use crate::hydro::schedule::check::check_schedule;
use crate::repository::{
    models::{irrigation_event::IrrigationEvent, irrigation_schedule::IrrigationSchedule},
    Repo,
};

use self::run::run_irrigation_event;
use super::irrigator::Irrigator;

/// Represents an IrrigationSchedule and its most recent IrrigationEvent
#[derive(Clone, Debug, PartialEq)]
pub struct ScheduleStatus {
    pub schedule: IrrigationSchedule,
    pub last_event: Option<IrrigationEvent>,
}

/// Intended to be run at startup and with a static lifetime. The process created
/// by this function will run on a synchronous tick. With each tick, the process
/// will queue any eligible events and run the next queued in a FIFO order.
///
///  # Arguments
///
///  * `db` - Handle to the database pool
///  * `sump` - Instance of the Sump object for running IrrigationEvents
///
pub fn start(repo: Repo, irrigator: Irrigator, frequency_sec: u64) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            let statuses = match check_schedule(repo).await {
                Ok(status) => status,
                Err(e) => {
                    tracing::error!("Could not check schedule: {}", e);
                    continue;
                }
            };

            let schedules = statuses
                .iter()
                .map(|status| status.schedule.clone())
                .collect();

            if let Err(e) = repo.queue_irrigation_events(schedules).await {
                tracing::error!("Could not create irrigation events: {}", e);
                continue;
            }

            let irrigator = irrigator.clone();
            run_irrigation_event(repo, &irrigator).await;
            sleep(Duration::from_secs(frequency_sec)).await;
        }
    })
}
