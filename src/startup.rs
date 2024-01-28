use actix_identity::IdentityMiddleware;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie, dev::Server, web, web::Data, App, HttpServer};
use actix_web::{error::ErrorBadRequest, web::JsonConfig};
use actix_web_opentelemetry::RequestTracing;
use serde_json::json;
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
        let address = format!("{}:{}", settings.server.host, settings.server.port);
        let listener =
            std::net::TcpListener::bind(address).expect("Could not bind server address.");
        let port = listener
            .local_addr()
            .expect("Could not get server address.")
            .port();

        let rt = tokio::runtime::Runtime::new().expect("Could not create runtime");
        let handle = rt.handle().clone();

        let hydro =
            Hydro::new(&settings.hydro, handle, gpio, repo).expect("Could not create hydro object");

        // TODO: fix clones
        //let db_clone = db_pool.clone();
        let server = HttpServer::new(move || {
            //let db_clone = db_clone.clone();
            //let hydro_clone = hydro.clone();
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
                .app_data(Data::new(repo))
                .app_data(Data::new(Some(hydro.clone())));

            app
        })
        .listen(listener)
        .expect(&format!("Could not listen on port {}", port))
        .run();

        Application {
            server,
            port,
            repo,
            _rt: rt,
        }
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
