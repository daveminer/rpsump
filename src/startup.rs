use actix_identity::IdentityMiddleware;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie, dev::Server, web, web::Data, App, HttpServer};
use actix_web::{error::ErrorBadRequest, web::JsonConfig};
use actix_web_opentelemetry::RequestTracing;
use serde_json::json;
use std::sync::Arc;

use crate::config::Settings;

use crate::controllers::{
    auth::auth_routes, info::info, irrigation::irrigation_routes, sump_event::sump_event,
};
use crate::database::DbPool;
use crate::sump::sensor::{listen_to_high_sensor, listen_to_low_sensor};
use crate::sump::{schedule, spawn_reporting_thread, Sump};

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub fn build(settings: Settings, db_pool: DbPool) -> Application {
        let address = format!("{}:{}", settings.server.host, settings.server.port);
        let listener =
            std::net::TcpListener::bind(address).expect("Could not bind server address.");
        let port = listener
            .local_addr()
            .expect("Could not get server address.")
            .port();
        let delay = match settings.clone().sump {
            Some(sump) => sump.pump_shutoff_delay,
            None => 0,
        };

        let irrigation = settings.clone().irrigation;

        if irrigation.enabled {
            schedule::start(
                db_pool.clone(),
                &settings.mailer.server_url,
                &settings.mailer.error_contact,
                &settings.mailer.auth_token,
            );
        }

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
            if settings.console.report_freq_secs > 0 {
                spawn_reporting_thread(
                    Arc::clone(&sump_clone.sensor_state),
                    settings.console.report_freq_secs,
                );
            }
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
                // HTTP API Routes
                .service(info)
                .service(sump_event)
                .service(web::scope("/auth").configure(auth_routes))
                .service(web::scope("/irrigation").configure(irrigation_routes))
                // Application configuration
                .app_data(Self::json_cfg())
                .app_data(Data::new(settings.clone()))
                .app_data(Data::new(db_pool.clone()));

            // Initialize the sump if enabled in configuration
            if sump.is_some() {
                app = app.app_data(Data::new(sump.clone()));
            }

            app
        })
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

    fn json_cfg() -> JsonConfig {
        web::JsonConfig::default().error_handler(|err, _req| {
            ErrorBadRequest(json!({
                "message": err.to_string()
            }))
        })
    }
}
