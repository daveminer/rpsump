use diesel_migrations::MigrationHarness;
use lazy_static::lazy_static;
use std::net::TcpListener;
use std::sync::Once;

use crate::config::Settings;
use crate::database::{new_pool, DbPool};

pub const MIGRATIONS: diesel_migrations::EmbeddedMigrations =
    diesel_migrations::embed_migrations!("migrations/");

pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub db_pool: DbPool,
    //pub email_server: MockServer,
    //pub test_user: TestUser,
    pub api_client: reqwest::Client,
    //pub email_client: EmailClient,
}

static INIT: Once = Once::new();
async fn setup_database() {
    INIT.call_once(|| {
        let connection = new_pool("test.db");
        connection.run_pending_migrations(MIGRATIONS)?;
    });
}

impl TestApp {
    pub async fn post_login<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.api_client
            .post(&format!("{}/auth/login", &self.address))
            .form(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_logout(&self) -> reqwest::Response {
        self.api_client
            .post(&format!("{}/auth/logout", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

pub async fn spawn_app() -> TestApp {
    //Lazy::force(&TRACING);

    // Launch a mock server to stand in for SendInBlue
    //let email_server = MockServer::start().await;

    // Randomise configuration to ensure test isolation
    // let configuration = {
    //     let mut c = get_configuration().expect("Failed to read configuration.");
    //     // Use a different database for each test case
    //     c.database.database_name = Uuid::new_v4().to_string();
    //     // Use a random OS port
    //     c.application.port = 0;
    //     // Use the mock server as email API
    //     c.email_client.base_url = email_server.uri();
    //     c
    // };

    let settings = Settings::new();

    // Create and migrate the database
    configure_database(&configuration.database).await;

    // Launch the application as a background task
    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application.");
    let application_port = application.port();
    let _ = tokio::spawn(application.run_until_stopped());

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .cookie_store(true)
        .build()
        .unwrap();

    let test_app = TestApp {
        address: format!("http://localhost:{}", application_port),
        port: application_port,
        db_pool: setup_database(),
        //email_server,
        //test_user: TestUser::generate(),
        api_client: client,
        //email_client: configuration.email_client.client(),
    };

    test_app.test_user.store(&test_app.db_pool).await;

    test_app
}
