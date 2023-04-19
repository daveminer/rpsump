use actix_web::error;
use actix_web::{get, web, web::Data, App, HttpResponse, HttpServer, Responder, Result};
use diesel::RunQueryDsl;
use std::thread;
use std::time::{Duration, Instant};

use crate::config::Settings;
use crate::database::{new_pool, DbPool};
use crate::models::sump_event::SumpEvent;
use crate::sump::Sump;

mod config;
mod database;
pub mod models {
    pub mod sump_event;
}
pub mod schema;
mod sump;

#[get("/info")]
async fn info(_req_body: String, sump: Data<Sump>) -> impl Responder {
    let body = serde_json::to_string(&sump.sensors()).expect("Could not serialize the pin state");

    HttpResponse::Ok().body(body)
}

#[get("/sump_event")]
async fn sump_event(_req_body: String, db: Data<DbPool>) -> Result<impl Responder> {
    let events = web::block(move || {
        let mut conn = database::conn(db);
        SumpEvent::all().load::<SumpEvent>(&mut conn)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body(format!("{:?}", events)))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let settings = Settings::new().expect("Environment configuration error.");
    let db_pool = new_pool(settings.clone().database()).expect("Could not initialize database.");
    let sump = Sump::new(db_pool.clone()).expect("Could not create sump object");
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
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
