use actix_cors::Cors;
use actix_identity::IdentityMiddleware;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie, dev::Server, http, web, web::Data, App, HttpServer};
use actix_web::{error::ErrorBadRequest, web::JsonConfig};
use actix_web_opentelemetry::RequestTracing;
use lazy_static::lazy_static;
use serde_json::json;
use std::net::TcpListener;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

use crate::config::Settings;
use crate::controllers::{
    auth::auth_routes, garden::garden_routes, heater::heater, info::info,
    pool_pump::pool_pump, sump_event::sump_event,
};

use crate::hydro::{gpio::Gpio, Hydro};
use crate::repository::Repo;

lazy_static! {
    static ref HYDRO_RT: Runtime = Runtime::new().expect("Failed to initialize runtime");
}

pub struct Application {
    port: u16,
    pub repo: Repo,
    server: Server,
}

impl Application {
    pub fn build(settings: Settings, gpio: &dyn Gpio, repo: Repo) -> Application {
        // Web server configuration
        let (_address, port, tcp_listener) = web_server_config(&settings);

        let handle = HYDRO_RT.handle();

        let hydro = Hydro::new(&settings.hydro, handle.clone(), gpio, repo)
            .expect("Could not create hydro object");

        let hydro_data = Data::new(Mutex::new(hydro));
        let repo_data = Data::new(repo);
        let settings_data = Data::new(settings.clone());
        let session_key = cookie::Key::generate();

        let server = HttpServer::new(move || {
            let allow_localhost = settings.server.allow_localhost_cors;
            let allowed_origins = settings.server.allowed_origins.clone();

            let cors = Cors::default()
                .allowed_origin_fn(move |origin, _req_head| {
                    let Ok(origin) = origin.to_str() else {
                        return false;
                    };

                    if allow_localhost && is_localhost_origin(origin) {
                        return true;
                    }

                    allowed_origins.iter().any(|allowed| allowed == origin)
                })
                .allowed_methods(vec!["GET", "POST", "PATCH", "DELETE", "OPTIONS"])
                .allowed_headers(vec![
                    http::header::AUTHORIZATION,
                    http::header::ACCEPT,
                    http::header::CONTENT_TYPE,
                ])
                .supports_credentials()
                .max_age(3600);

            App::new()
                .wrap(cors)
                .wrap(RequestTracing::new())
                // Session tools
                .wrap(IdentityMiddleware::default())
                .wrap(SessionMiddleware::new(
                    CookieSessionStore::default(),
                    session_key.clone(),
                ))
                // HTTP API Routes
                .service(heater)
                .service(info)
                .service(pool_pump)
                .service(sump_event)
                .service(web::scope("/auth").configure(auth_routes))
                .service(web::scope("/garden").configure(garden_routes))
                // Application configuration
                .app_data(JsonConfig::default().error_handler(|err, _req| {
                    ErrorBadRequest(json!({
                        "message": err.to_string()
                    }))
                }))
                .app_data(settings_data.clone())
                .app_data(repo_data.clone())
                .app_data(hydro_data.clone())
        })
        .listen(tcp_listener)
        .unwrap_or_else(|_| panic!("Could not listen on port {}", port))
        .run();

        Application { server, port, repo }
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

/// Matches a development origin by host, so a hostname that merely contains
/// "localhost" (`https://localhost.attacker.com`) is not treated as local.
fn is_localhost_origin(origin: &str) -> bool {
    let Some((scheme, rest)) = origin.split_once("://") else {
        return false;
    };

    if scheme != "http" && scheme != "https" {
        return false;
    }

    let host = match rest.rsplit_once(':') {
        // An IPv6 host keeps its brackets; only a trailing :port is stripped.
        Some((host, port)) if port.chars().all(|c| c.is_ascii_digit()) => host,
        _ => rest,
    };

    matches!(host, "localhost" | "127.0.0.1" | "[::1]")
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

#[cfg(test)]
mod tests {
    use super::is_localhost_origin;

    #[test]
    fn accepts_local_development_origins() {
        assert!(is_localhost_origin("http://localhost"));
        assert!(is_localhost_origin("http://localhost:5173"));
        assert!(is_localhost_origin("https://localhost:5173"));
        assert!(is_localhost_origin("http://127.0.0.1:8080"));
        assert!(is_localhost_origin("http://[::1]:8080"));
    }

    #[test]
    fn rejects_hosts_that_only_look_local() {
        assert!(!is_localhost_origin("https://localhost.attacker.com"));
        assert!(!is_localhost_origin("https://not-localhost:5173"));
        assert!(!is_localhost_origin("http://evil.com/#localhost"));
        assert!(!is_localhost_origin("localhost"));
    }
}
