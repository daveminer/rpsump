use crate::hydro::Irrigator;
use crate::repository::Repo;
use anyhow::Error;

use super::due_statuses;
use super::run::run_next_event;

pub(crate) async fn check_schedule(repo: Repo, irrigator: Irrigator) -> Result<(), Error> {
    // Get the statuses of all the schedules
    let statuses = repo.schedule_statuses().await?;

    // Determine which statuses are due to run
    let events_to_insert = due_statuses(statuses, chrono::Utc::now().naive_utc());

    repo.queue_irrigation_events(events_to_insert).await?;

    // Run irrigation events
    tokio::task::spawn_blocking(move || run_next_event(repo, irrigator));

    Ok(())
}
