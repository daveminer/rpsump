use std::sync::Mutex;

use actix_cors::Cors;
use actix_identity::IdentityMiddleware;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie, dev::Server, web, web::Data, App, HttpServer};
use actix_web::{error::ErrorBadRequest, web::JsonConfig};
use actix_web_opentelemetry::RequestTracing;
use serde_json::json;
use std::net::TcpListener;
use tokio::runtime::Runtime;

use crate::config::Settings;
use crate::controllers::{
    auth::auth_routes, heater::heater, info::info, irrigation::irrigation_routes,
    pool_pump::pool_pump, sump_event::sump_event,
};

use crate::hydro::{gpio::Gpio, Hydro};
use crate::repository::Repo;

pub struct Application {
    port: u16,
    pub repo: Repo,
    _rt: Runtime,
    server: Server,
}

impl Application {
    pub fn build<G>(settings: Settings, gpio: &G, repo: Repo) -> Application
    where
        G: Gpio,
    {
        // Web server configuration
        let (_address, port, tcp_listener) = web_server_config(&settings);

        let hydro_rt = tokio::runtime::Runtime::new().expect("Could not create runtime");
        let handle = hydro_rt.handle().clone();

        let hydro =
            Hydro::new(&settings.hydro, handle, gpio, repo).expect("Could not create hydro object");

        let server = HttpServer::new(move || {
            let mut cors = Cors::default();

            if settings.server.allow_localhost_cors {
                cors = cors
                    .allowed_origin_fn(|origin, _req_head| origin.as_bytes().contains("localhost"));
            }

            cors.allowed_methods(vec!["GET", "OPTION", "POST"])
                .allowed_headers(vec![
                    http::header::AUTHORIZATION,
                    http::header::ACCEPT,
                    http::header::CONTENT_TYPE,
                ])
                .supports_credentials()
                .max_age(3600);

            let app = App::new()
                .wrap(cors)
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
                .app_data(JsonConfig::default().error_handler(|err, _req| {
                    ErrorBadRequest(json!({
                        "message": err.to_string()
                    }))
                }))
                .app_data(Data::new(settings.clone()))
                .app_data(Data::new(repo))
                .app_data(Data::new(Mutex::new(Some(hydro.clone()))));

            app
        })
        .listen(tcp_listener)
        .expect(&format!("Could not listen on port {}", port))
        .run();

        Application {
            server,
            port,
            repo,
            // Keep this runtime for the lifetime of Application
            _rt: hydro_rt,
        }
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

fn web_server_config(settings: &Settings) -> (String, u16, TcpListener) {
    let address = format!("{}:{}", settings.server.host, settings.server.port);
    let address_clone = address.clone();
    let listener = std::net::TcpListener::bind(address).expect("Could not bind server address.");

    let port = listener
        .local_addr()
        .expect("Could not get server address.")
        .port();

    (address_clone, port, listener)
}
