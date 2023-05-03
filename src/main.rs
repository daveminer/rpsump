use crate::config::Settings;
use crate::controllers::auth::auth_routes;
use crate::controllers::{info::info, sump_event::sump_event};
use crate::database::new_pool;
use crate::sump::Sump;
use actix_identity::IdentityMiddleware;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie, dev::Service, web, web::Data, App, HttpServer};
use actix_web_opentelemetry::RequestTracing;
use middleware::telemetry;
use opentelemetry::{
    global,
    trace::{FutureExt, TraceContextExt, Tracer},
    Key,
};
use std::process::exit;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

pub mod auth;
mod config;
mod controllers;
mod database;
mod email;
mod middleware;
pub mod models;
pub mod schema;
mod sump;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    ctrlc::set_handler(move || {
        println!("received Ctrl+C!");
        exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    let settings = Settings::new().expect("Environment configuration error.");
    let db_pool = new_pool(&settings.database_url).expect("Could not initialize database.");

    // let sump = Sump::new(db_pool.clone(), &settings.sump).expect("Could not create sump object");
    // let sump_clone = sump.clone();

    // let settings_clone = settings.clone();
    // let _sync_reporter_thread = thread::spawn(move || {
    //     let mut start_time = Instant::now();

    //     loop {
    //         // Report to console
    //         println!("{:?}", &sump_clone.sensors());

    //         // Wait for N seconds
    //         let elapsed_time = start_time.elapsed();
    //         if elapsed_time < Duration::from_secs(settings_clone.console.report_freq_secs) {
    //             thread::sleep(
    //                 Duration::from_secs(settings_clone.console.report_freq_secs) - elapsed_time,
    //             );
    //         }
    //         start_time = Instant::now();
    //     }
    // });

    //env_logger::init();
    telemetry::init_tracer(&settings).expect("Could not initialize telemetry.");

    HttpServer::new(move || {
        App::new()
            // Telemetry
            // .wrap_fn(|req, srv| {
            //     let tracer = global::tracer("request");
            //     tracer.in_span("middleware", move |cx| {
            //         cx.span()
            //             .set_attribute(Key::new("path").string(req.path().to_string()));
            //         srv.call(req).with_context(cx)
            //     })
            // })
            .wrap(RequestTracing::new())
            // Session tools
            .wrap(IdentityMiddleware::default())
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                cookie::Key::generate(),
            ))
            // Rate limiter
            .wrap(middleware::rate_limiter::new(
                settings.rate_limiter.per_second,
                settings.rate_limiter.burst_size,
            ))
            // Application configuration
            .app_data(Data::new(settings.clone()))
            .app_data(Data::new(db_pool.clone()))
            //.app_data(Data::new(sump.clone()))
            // HTTP API Routes
            .service(info)
            .service(sump_event)
            .service(web::scope("/auth").configure(auth_routes))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    // Ensure all spans have been reported
    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}
