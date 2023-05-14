use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use once_cell::sync::Lazy;
use uuid::Uuid;

use rpsump::config::Settings;
use rpsump::database::{new_pool, DbPool};
use rpsump::startup::Application;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub db_pool: DbPool,
    //pub email_server: MockServer,
    //pub test_user: TestUser,
    pub api_client: reqwest::Client,
    //pub email_client: EmailClient,
}

static DB_INIT: Lazy<DbPool> = Lazy::new(|| {
    let pool = new_pool(&"test.db".to_string());
    let mut conn = pool.get().expect("Could not get connection.");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Could not run migrations.");

    pool
});

impl TestApp {
    pub async fn post_login<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.api_client
            .post(&format!("{}/auth/login", &self.address))
            .json(body)
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

    let mut settings = Settings::new();
    settings.database_url = Uuid::new_v4().to_string();
    settings.server.port = 0;

    let db_pool = Lazy::force(&DB_INIT);
    let application = Application::build(settings, db_pool.clone());
    let port = application.port();

    let _ = tokio::spawn(application.run_until_stopped());

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        //.cookie_store(true)
        .build()
        .unwrap();

    let test_app = TestApp {
        address: format!("http://localhost:{}", port),
        port: port,
        db_pool: db_pool.clone(),
        //email_server,
        //test_user: TestUser::generate(),
        api_client: client,
        //email_client: configuration.email_client.client(),
    };

    //test_app.test_user.store(&test_app.db_pool).await;

    test_app
}
