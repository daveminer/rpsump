pub mod check;
pub mod run;

use chrono::{Datelike, NaiveDateTime};
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

use crate::hydro::schedule::check::check_schedule;
use crate::repository::{
    models::{irrigation_event::IrrigationEvent, irrigation_schedule::IrrigationSchedule},
    Repo,
};

use super::irrigator::Irrigator;

/// Represents an IrrigationSchedule and its most recent IrrigationEvent
#[derive(Clone, Debug, PartialEq)]
pub struct Status {
    pub schedule: IrrigationSchedule,
    pub last_event: Option<IrrigationEvent>,
}

/// Intended to be run at startup and with a static lifetime. The process created
/// by this function will run on a synchronous tick. The tick will queue and
/// run events in a FIFO order.
///
///  # Arguments
///
///  * `db` - Handle to the database pool
///  * `sump` - Instance of the Sump object for running IrrigationEvents
///
pub fn start(repo: Repo, irrigator: Irrigator, frequency_ms: u64) -> JoinHandle<()> {
    tokio::spawn(async move {
        // TODO: loop
        if let Err(e) = check_schedule(repo, irrigator).await {
            tracing::error!("Could not check schedule: {}", e);
        }
        sleep(Duration::from_millis(frequency_ms)).await;
    })
}

fn due_statuses(status_list: Vec<Status>, now: NaiveDateTime) -> Vec<Status> {
    let mut schedules_to_run = status_list
        .into_iter()
        // Schedule is active
        .filter(|status| status.schedule.active == true)
        // Schedule is for today
        .filter(|status| {
            status
                .schedule
                .days_of_week
                .contains(&now.weekday().to_string())
        })
        // Schedule's run time has passed
        .filter(|status| status.schedule.start_time < now.time())
        // Schedule has not been queued already today
        .filter(|status| {
            if status.last_event.is_none() {
                return true;
            }

            let last_event = status.last_event.clone().unwrap();
            last_event.created_at.date() != now.date()
        })
        .collect::<Vec<Status>>();

    schedules_to_run.sort_by(|a, b| a.schedule.start_time.cmp(&b.schedule.start_time));

    return schedules_to_run;
}

#[cfg(test)]
mod tests {}
