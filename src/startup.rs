use actix_identity::IdentityMiddleware;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie, dev::Server, web, web::Data, App, HttpServer};
use actix_web::{error::ErrorBadRequest, web::JsonConfig};
use actix_web_opentelemetry::RequestTracing;
use serde_json::json;

use crate::config::Settings;

use crate::controllers::{
    auth::auth_routes, heater::heater, info::info, irrigation::irrigation_routes,
    pool_pump::pool_pump, sump_event::sump_event,
};
use crate::database::DbPool;
use crate::hydro::{gpio::Gpio, Hydro};
//use crate::hydro::sensor::{listen_to_high_sensor, listen_to_low_sensor};

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub fn build<G>(settings: Settings, db_pool: &DbPool, gpio: &G) -> Application
    where
        G: Gpio,
    {
        // Web server configuration
        let address = format!("{}:{}", settings.server.host, settings.server.port);
        let listener =
            std::net::TcpListener::bind(address).expect("Could not bind server address.");
        let port = listener
            .local_addr()
            .expect("Could not get server address.")
            .port();

        // TODO: Handlers
        let low_sensor_handler = |level| {
            tracing::info!(
                target = module_path!(),
                "Low sensor state changed to {:?}",
                level
            );
        };

        let hydro = Hydro::new(
            db_pool,
            &settings.hydro,
            gpio,
            Box::new(low_sensor_handler),
            Box::new(low_sensor_handler),
            Box::new(low_sensor_handler),
        )
        .expect("Could not create hydro object");

        // Pump shutoff delay
        // let delay = match settings.clone().sump {
        //     Some(sump) => sump.pump_shutoff_delay,
        //     None => 0,
        // };

        // let sump = match settings.clone().sump {
        //     None => None,
        //     Some(sump_config) => Some(
        //         Sump::new(db_pool.clone(), &sump_config).expect("Could not create sump object"),
        //     ),
        // };

        // let sump_clone = sump.clone();

        // if sump_clone.is_some() {
        //     let sump_clone = sump_clone.unwrap();

        //     listen_to_high_sensor(
        //         Arc::clone(&sump_clone.high_sensor_pin),
        //         Arc::clone(&sump_clone.pump_control_pin),
        //         Arc::clone(&sump_clone.sensor_state),
        //         db_pool.clone(),
        //     );

        //     listen_to_low_sensor(
        //         Arc::clone(&sump_clone.low_sensor_pin),
        //         Arc::clone(&sump_clone.pump_control_pin),
        //         Arc::clone(&sump_clone.sensor_state),
        //         delay,
        //         db_pool.clone(),
        //     );
        //     if settings.console.report_freq_secs > 0 {
        //         spawn_reporting_thread(
        //             Arc::clone(&sump_clone.sensor_state),
        //             settings.console.report_freq_secs,
        //         );
        //     }

        //     if sump_clone.irrigation_enabled {
        //         schedule::start(
        //             db_pool.clone(),
        //             sump_clone,
        //             settings
        //                 .clone()
        //                 .sump
        //                 .unwrap()
        //                 .irrigation
        //                 .process_frequency_ms,
        //         );
        //     }
        // }

        // TODO: fix clones
        let db_clone = db_pool.clone();
        let server = HttpServer::new(move || {
            let db_clone = db_clone.clone();
            let hydro_clone = hydro.clone();
            let app = App::new()
                .wrap(RequestTracing::new())
                // Session tools
                .wrap(IdentityMiddleware::default())
                .wrap(SessionMiddleware::new(
                    CookieSessionStore::default(),
                    cookie::Key::generate(),
                ))
                // HTTP API Routes
                .service(heater)
                .service(info)
                .service(pool_pump)
                .service(sump_event)
                .service(web::scope("/auth").configure(auth_routes))
                .service(web::scope("/irrigation").configure(irrigation_routes))
                // Application configuration
                .app_data(Self::json_cfg())
                .app_data(Data::new(settings.clone()))
                .app_data(Data::new(db_clone))
                .app_data(Data::new(Some(hydro_clone)));

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
