use std::sync::{Arc, Mutex};

use crate::hydro::debounce::Debouncer;
use crate::hydro::{control::Output, Control, Level};
use crate::repository::models::sump_event::SumpEvent;
use crate::repository::Repo;

pub fn handler(
    level: Level,
    handler: dyn FnOnce(Level, Control, Repo) + Send + 'static,
    shared_debouncer: Arc<Mutex<Option<Debouncer>>>,
    rt: &mut tokio::runtime::Runtime,
) {
    let deb = Arc::clone(&shared_debouncer);
    let mut deb_lock = deb.lock().unwrap();

    if deb_lock.is_some() {
        deb_lock.as_mut().unwrap().reset_deadline(level);
        return;
    }

    let sleep = deb_lock.as_ref().unwrap().sleep();

    rt.block_on(async {
        sleep.await;
        *deb_lock = None;
        drop(deb_lock);

        //handler(level).await;
    });
}

// pub fn handler(
//     level: Level,
//     handler: impl FnOnce(Level, Control, DbPool) + Send + 'static,
//     shared_debouncer: Arc<Mutex<Option<Debouncer>>>,
//     pump: Control,
//     db: DbPool,
//     rt: &mut tokio::runtime::Runtime,
// ) {
//     let deb = Arc::clone(&shared_debouncer);
//     let mut deb_lock = deb.lock().unwrap();

//     if deb_lock.is_some() {
//         deb_lock.as_mut().unwrap().reset_deadline(level);
//         return;
//     }

//     let sleep = deb_lock.as_ref().unwrap().sleep();

//     rt.block_on(async {
//         sleep.await;
//         *deb_lock = None;
//         drop(deb_lock);

//         update_sensor(level, pump, db.clone()).await;
//     });
// }

#[tracing::instrument(skip(repo))]
pub async fn update_sensor(level: Level, mut pump: Control, repo: Repo) {
    // Turn the pump on
    if level == Level::High {
        pump.on();

        tracing::info!("Sump pump turned on.");

        if let Err(e) =
            SumpEvent::create("pump on".to_string(), "reservoir full".to_string(), repo).await
        {
            tracing::error!(
                target = module_path!(),
                error = e.to_string(),
                "Failed to create sump event for pump on"
            );
        }
    }
}
