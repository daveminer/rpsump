use futures::executor::block_on;

use crate::database::DbPool;
use crate::hydro::schedule::get_schedule_statuses;
use crate::hydro::Irrigator;
use crate::models::irrigation_event::IrrigationEvent;

use super::due_statuses;
use super::run::run_next_event;

pub(crate) fn check_schedule(db: DbPool, irrigator: Irrigator) {
    // Get the statuses of all the schedules
    let statuses = match get_schedule_statuses(db.clone()) {
        Ok(statuses) => statuses,
        Err(e) => {
            tracing::error!("Could not get schedule statuses: {}", e);

            return;
        }
    };

    // Determine which statuses are due to run
    let events_to_insert = due_statuses(statuses, chrono::Utc::now().naive_utc());

    // Insert a queued event for each due status
    events_to_insert.into_iter().for_each(|status| {
        if let Err(e) = block_on(IrrigationEvent::create_irrigation_events_for_status(
            db.clone(),
            status,
        )) {
            tracing::error!("Could not insert irrigation event: {}", e)
        }
    });

    // Run irrigation events
    block_on(run_next_event(db, irrigator));
}
