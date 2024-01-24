use tokio::time::{sleep, Duration};

use crate::hydro::control::Output;
use crate::hydro::{Control, Level};
use crate::repository::Repo;

#[tracing::instrument(skip(repo))]
pub async fn update_sensor(level: Level, mut pump: Control, repo: Repo, delay: u64) {
    // Turn the pump off
    if level == Level::Low {
        if delay > 0 {
            sleep(Duration::from_secs(delay as u64)).await;
        }

        pump.off();
        tracing::info!(target = module_path!(), "Sump pump turned off");

        if let Err(e) = repo
            .create_sump_event("pump_off".to_string(), "reservoir empty".to_string())
            .await
        {
            tracing::error!(
                target = module_path!(),
                error = e.to_string(),
                "Failed to create sump event for pump off"
            );
        }
    }
}
