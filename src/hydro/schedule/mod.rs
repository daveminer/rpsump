pub mod check;
pub mod run;

use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

use crate::hydro::schedule::check::check_schedule;
use crate::repository::{
    models::{irrigation_event::IrrigationEvent, irrigation_schedule::IrrigationSchedule},
    Repo,
};
use crate::util::spawn_blocking_with_tracing;

use self::run::run_next_event;
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
pub fn start(repo: Repo, irrigator: Irrigator, frequency_sec: u64) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            println!("Looping...");
            if let Err(e) = check_schedule(repo).await {
                tracing::error!("Could not check schedule: {}", e);
            }
            let irrigator = irrigator.clone();
            let _handle =
                spawn_blocking_with_tracing(move || run_next_event(repo, irrigator)).await;
            sleep(Duration::from_secs(frequency_sec)).await;
        }
    })
}
