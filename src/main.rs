#[macro_use]
extern crate diesel_migrations;

use actix_identity::IdentityMiddleware;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie, dev::Server, web, web::Data, App, HttpServer};

use actix_web_opentelemetry::RequestTracing;
use middleware::telemetry;
use std::process::exit;

use crate::config::Settings;
use crate::controllers::auth::auth_routes;
use crate::controllers::{info::info, sump_event::sump_event};
use crate::database::{new_pool, DbPool};
use crate::sump::Sump;

pub mod auth;
mod config;
pub mod controllers;
mod database;
mod email;
mod middleware;
pub mod models;
pub mod schema;
mod sump;

#[cfg(test)]
mod tests;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_exit_handler();

    let settings = Settings::new();
    let db_pool = new_pool(&settings.database_url);
    telemetry::init_tracer(&settings).expect("Could not initialize telemetry.");

    let _ = build_server(settings, db_pool).await;

    Ok(())
}

fn build_server(settings: Settings, db_pool: DbPool) -> Server {
    HttpServer::new(move || {
        let mut app = App::new()
            .wrap(RequestTracing::new())
            // Session tools
            .wrap(IdentityMiddleware::default())
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                cookie::Key::generate(),
            ))
            // Rate limiter
            .wrap(middleware::rate_limiter::new(
                settings.rate_limiter.per_second.clone(),
                settings.rate_limiter.burst_size.clone(),
            ))
            // HTTP API Routes
            .service(info)
            .service(sump_event)
            .service(web::scope("/auth").configure(auth_routes))
            // Application configuration
            .app_data(Data::new(settings.clone()))
            .app_data(Data::new(db_pool.clone()));

        // Initialize the sump if enabled in configuration
        if settings.sump.is_some() {
            let sump = Sump::new(db_pool.clone(), settings.sump.as_ref().unwrap())
                .expect("Could not create sump object");

            sump.spawn_reporting_thread(settings.console.report_freq_secs);

            app = app.app_data(Data::new(sump.clone()));
        }

        app
    })
    .bind(("127.0.0.1", 8080))
    .expect("Could not bind to port 8080")
    .run()
}

// actix-web will handle signals to exit but doesn't offer a hook to customize it
fn init_exit_handler() {
    ctrlc::set_handler(move || {
        // Ensure all spans have been reported
        opentelemetry::global::shutdown_tracer_provider();

        exit(0);
    })
    .expect("Error setting Ctrl-C handler");
}
