use serde::Deserialize;

use crate::hydro::{control::Output, Control, Level};
use crate::repository::Repo;

#[tracing::instrument(skip(repo))]
pub async fn update_sensor(level: Level, mut pump: Control, repo: Repo) {
    // Turn the pump on
    if level == Level::High {
        pump.on();

        tracing::info!("Sump pump turned on.");

        if let Err(e) = repo
            .create_sump_event("pump on".to_string(), "reservoir full".to_string())
            .await
        {
            tracing::error!(
                target = module_path!(),
                error = e.to_string(),
                "Failed to create sump event for pump on"
            );
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum PumpAction {
    On,
    Off,
}

#[tracing::instrument(skip(repo))]
pub async fn handle_sensor_signal(action: PumpAction, mut pump: Control, repo: Repo) {
    match action {
        PumpAction::On => pump.on().await,
        PumpAction::Off => pump.off().await,
    };

    tracing::info!("Sump pump turned {:?}.", action);

    if let Err(e) = repo
        .create_sump_event(format!("pump {:?}", action), "reservoir full".to_string())
        .await
    {
        tracing::error!(
            target = module_path!(),
            error = e.to_string(),
            "Failed to create sump event for pump on"
        );
    }
}
