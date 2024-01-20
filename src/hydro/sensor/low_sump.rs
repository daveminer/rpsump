use tokio::time::{sleep, Duration};

use crate::database::RealDbPool;
use crate::hydro::control::Output;
use crate::hydro::{Control, Level};
use crate::models::sump_event::SumpEvent;

#[tracing::instrument(skip(db))]
pub async fn update_sensor(level: Level, mut pump: Control, db: RealDbPool, delay: u64) {
    // Turn the pump on
    if level == Level::Low {
        if delay > 0 {
            sleep(Duration::from_secs(delay as u64)).await;
        }

        pump.off();
        tracing::info!(target = module_path!(), "Sump pump turned off");

        if let Err(e) =
            SumpEvent::create("pump off".to_string(), "reservoir empty".to_string(), db).await
        {
            tracing::error!(
                target = module_path!(),
                error = e.to_string(),
                "Failed to create sump event for pump off"
            );
        }
    }
}
