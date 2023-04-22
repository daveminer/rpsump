use crate::config::Settings;
use crate::controllers::auth::auth_routes;
use crate::controllers::info::info;
use crate::controllers::sump_event::sump_event;
use crate::database::new_pool;
use crate::sump::Sump;
use actix_web::{web, web::Data, App, HttpServer};
use std::thread;
use std::time::{Duration, Instant};

pub mod auth {
    pub mod authenticated_user;
    pub mod claim;
}
mod config;
mod controllers;
mod database;
pub mod models {
    pub mod sump_event;
    pub mod user;
}
pub mod schema;
mod sump;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let settings = Settings::new().expect("Environment configuration error.");
    let db_pool = new_pool(&settings.database_url).expect("Could not initialize database.");
    let sump = Sump::new(db_pool.clone(), &settings.sump).expect("Could not create sump object");
    let sump_clone = sump.clone();

    let settings_clone = settings.clone();
    let _sync_reporter_thread = thread::spawn(move || {
        let mut start_time = Instant::now();

        loop {
            // Report to console
            println!("{:?}", &sump_clone.sensors());

            // Wait for N seconds
            let elapsed_time = start_time.elapsed();
            if elapsed_time < Duration::from_secs(settings_clone.console.report_freq_secs) {
                thread::sleep(
                    Duration::from_secs(settings_clone.console.report_freq_secs) - elapsed_time,
                );
            }
            start_time = Instant::now();
        }
    });

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(db_pool.clone()))
            .app_data(Data::new(sump.clone()))
            .service(info)
            .service(sump_event)
            .service(web::scope("/api/auth").configure(auth_routes))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
