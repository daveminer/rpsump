use actix_identity::IdentityMiddleware;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie, dev::Server, web, web::Data, App, HttpServer};
use actix_web_opentelemetry::RequestTracing;
use std::sync::Arc;

use crate::config::Settings;
use crate::controllers::{auth::auth_routes, info::info, sump_event::sump_event};
use crate::database::DbPool;
use crate::middleware::rate_limiter;
use crate::sump::sensor::{listen_to_high_sensor, listen_to_low_sensor};
use crate::sump::{spawn_reporting_thread, Sump};

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub fn build(settings: Settings, db_pool: DbPool) -> Application {
        let address = format!("{}:{}", settings.server.host, settings.server.port);
        let listener =
            std::net::TcpListener::bind(address).expect("Could not bind server address.");
        let port = listener.local_addr().unwrap().port();
        let delay = settings.clone().sump.unwrap().pump_shutoff_delay;

        let sump = match settings.clone().sump {
            None => None,
            Some(sump_config) => Some(
                Sump::new(db_pool.clone(), &sump_config).expect("Could not create sump object"),
            ),
        };

        let sump_clone = sump.clone();

        if sump_clone.is_some() {
            let sump_clone = sump_clone.unwrap();

            listen_to_high_sensor(
                Arc::clone(&sump_clone.high_sensor_pin),
                Arc::clone(&sump_clone.pump_control_pin),
                Arc::clone(&sump_clone.sensor_state),
                db_pool.clone(),
            );

            listen_to_low_sensor(
                Arc::clone(&sump_clone.low_sensor_pin),
                Arc::clone(&sump_clone.pump_control_pin),
                Arc::clone(&sump_clone.sensor_state),
                delay,
                db_pool.clone(),
            );
        }

        let server = HttpServer::new(move || {
            let mut app = App::new()
                .wrap(RequestTracing::new())
                // Session tools
                .wrap(IdentityMiddleware::default())
                .wrap(SessionMiddleware::new(
                    CookieSessionStore::default(),
                    cookie::Key::generate(),
                ))
                // Rate limiter
                .wrap(rate_limiter::new(
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
            if sump.is_some() {
                let sump = sump.as_ref().unwrap();
                spawn_reporting_thread(
                    Arc::clone(&sump.sensor_state),
                    settings.console.report_freq_secs,
                );

                app = app.app_data(Data::new(sump.clone()));
            }

            app
        })
        //.bind(("127.0.0.1", 8080))
        .listen(listener)
        .expect(&format!("Could not listen on port {}", port))
        .run();

        Application { server, port }
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}
