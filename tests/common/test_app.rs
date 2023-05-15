use once_cell::sync::Lazy;
use rusqlite::{Connection, OpenFlags};
use std::fs::copy;
use tempfile::TempDir;
use uuid::Uuid;

use rpsump::config::Settings;
use rpsump::database::{new_pool, DbPool};
use rpsump::startup::Application;

pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub db_pool: DbPool,
    //pub email_server: MockServer,
    //pub test_user: TestUser,
    pub api_client: reqwest::Client,
    //pub email_client: EmailClient,
}

const DB_TEMPLATE_FILE: &str = "test.db";

static TEST_DB_TEMPLATE: Lazy<TempDir> = Lazy::new(|| {
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

    const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
    let temp_dir = TempDir::new().unwrap();
    let template_db_path = temp_dir.path().join(DB_TEMPLATE_FILE);

    let mut conn = new_pool(&template_db_path.to_str().unwrap().to_string())
        .get()
        .unwrap();
    conn.run_pending_migrations(MIGRATIONS)
        .expect("diesel migrations");

    temp_dir
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

pub fn spawn_test_db() -> String {
    let test_db_dir = Lazy::force(&TEST_DB_TEMPLATE);
    let test_db_path = test_db_dir.path().to_str().unwrap().to_string();
    let template_db = format!("{}/{}", test_db_path, DB_TEMPLATE_FILE);

    let db_instance_file = format!("{}.db", Uuid::new_v4().to_string());
    let db_instance = format!("{}/{}", test_db_path, db_instance_file);

    let _conn = Connection::open_with_flags(
        &db_instance,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
    )
    .unwrap();

    // copy
    copy(&template_db, &db_instance).unwrap();

    db_instance
}

pub async fn spawn_app() -> TestApp {
    let mut settings = Settings::new();
    settings.database_url = spawn_test_db();
    settings.server.port = 0;

    let db_pool = new_pool(&spawn_test_db());
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
