use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use anyhow::{anyhow, Error};
use diesel::r2d2::{self, ConnectionManager, Pool, PooledConnection};
use diesel::{Connection, SqliteConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use mockall::predicate::eq;
use once_cell::sync::Lazy;
use rpsump::hydro::gpio::{Gpio, Level, MockGpio, MockInputPin, MockOutputPin, MockPin};
use rpsump::repository::implementation::Implementation;
use serde_json::{json, Value};
use tempfile::TempDir;
use tokio::sync::OnceCell;
use wiremock::MockServer;

use rpsump::config::Settings;
use rpsump::repository::{self, implementation, Repo, Repository};
use rpsump::startup::Application;

// TODO: move to shared location
use crate::auth::authenticated_user::create_auth_header;

type DbConn = PooledConnection<ConnectionManager<SqliteConnection>>;
type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

const DB_TEMPLATE_FILE: &str = "rpsump_test.db";

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

static TEST_DB_POOL: Lazy<OnceCell<DbPool>> = Lazy::new(OnceCell::new);
static TEST_REPO: Lazy<OnceCell<implementation::Implementation>> = Lazy::new(OnceCell::new);

pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub repo: Repo,
    repo_temp_dir: TempDir,
    pub email_server: MockServer,
    pub api_client: reqwest::Client,
}

// static MIGRATED_DB_TEMPLATE: Lazy<PathBuf> = Lazy::new(|| {
//     // Create a file for the database
//     let temp_dir = TempDir::new().unwrap();
//     let db_path = temp_dir.path().join(DB_TEMPLATE_FILE);

//     let manager = ConnectionManager::<SqliteConnection>::new(db_path.to_str().unwrap());
//     // Initialize a repository with the database file and
//     // use it to run migrations.
//     let repo = implementation::Implementation::new(Some("/tmp/rpsump_test.db".to_string()))
//         .await
//         .expect("Couldn't get test repo");

//     let mut conn = repo.pool().await.unwrap().get().unwrap();

//     let _ = conn.run_pending_migrations(MIGRATIONS).unwrap();

//     // Return the path to the migrated template database
//     db_path
// });

static MIGRATED_DB_TEMPLATE: Lazy<OnceCell<(PathBuf, TempDir)>> = Lazy::new(OnceCell::new);

// Call this function at the start of your program or before you need MIGRATED_DB_TEMPLATE
async fn initialize_db_template() -> (PathBuf, TempDir) {
    // Create a file for the database
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join(DB_TEMPLATE_FILE);

    let manager = ConnectionManager::<SqliteConnection>::new(db_path.to_str().unwrap());
    let pool = Pool::new(manager).unwrap();
    let mut conn = pool.get().unwrap();

    let _ = conn.run_pending_migrations(MIGRATIONS).unwrap();

    // Return the path to the migrated template database
    // TODO: check if temp_dir lifetime is needed
    (db_path, temp_dir)
}

// pub async fn initialize() -> Result<(), Box<dyn std::error::Error>> {
//     TEST_APP.get_or_init(|| async { spawn_app().await }).await;
//     Ok(())
// }

pub async fn migrated_pathbuf() -> (PathBuf, TempDir) {
    // MIGRATED_DB_TEMPLATE
    //     .get_or_init(initialize_db_template)
    //     .await;

    let test_db_dir = TempDir::new().unwrap();

    // Create new file for the test app database
    let test_db_path = test_db_dir.path().join(DB_TEMPLATE_FILE);

    // Get the PathBuf from the OnceCell
    let (template_path, _temp_dir) = MIGRATED_DB_TEMPLATE
        .get_or_init(initialize_db_template)
        .await;

    // Create a new file at template_path
    File::create(&test_db_path).expect("Failed to create template file");

    // Copy the migrated template database to the new file
    fs::copy(template_path, &test_db_path).expect("Failed to copy template database");

    (test_db_path, test_db_dir)
}

// pub async fn test_repo_clone(test_repo: Implementation) -> Implementation {

//     let pool = repo.pool().await.unwrap();
//     let mut conn = pool.get().unwrap();

//     let _ = conn.run_pending_migrations(MIGRATIONS).unwrap();

//     repo
// }

// pub async fn repo() -> &'static Implementation {
//     TEST_REPO
//         .get_or_init(|| async {
//             // Delete the old database file, if it exists
//             let _ = fs::remove_file("/tmp/rpsump_test.db");

//             let repo = implementation::Implementation::new(Some("/tmp/rpsump_test.db".to_string()))
//                 .await
//                 .expect("Couldn't get test repo");

//             let mut conn = repo.pool().await.unwrap().get().unwrap();

//             let _ = conn.run_pending_migrations(MIGRATIONS).unwrap();

//             repo
//         })
//         .await
// }

// async fn setup_database(reop) -> Result<DbPool, Error> {
//     let manager = ConnectionManager::<SqliteConnection>::new("/tmp/rpsump_test.db");
//     let pool = r2d2::Pool::builder().build(manager)?;

//     // Get a connection from the pool
//     let mut conn = pool.get()?;

//     let result = conn
//         .run_pending_migrations(MIGRATIONS)
//         .map_err(|e| anyhow!(e.to_string()))?;

//     println!("RESULT: {:?}", result);

//     Ok(pool)
// }

// pub async fn initialize_test_db() -> Result<(), Box<dyn std::error::Error>> {
//     TEST_DB_POOL
//         .get_or_init(|| async { setup_database().await.expect("Failed to setup database") })
//         .await;
//     Ok(())
// }

static GPIO: Lazy<OnceCell<MockGpio>> = Lazy::new(OnceCell::new);

pub async fn get_gpio() -> Result<&'static MockGpio, Error> {
    Ok(GPIO.get_or_init(|| async { spawn_test_gpio() }).await)
}

impl TestApp {
    pub async fn delete_irrigation_schedule(&self, token: String, id: i32) -> reqwest::Response {
        let (header_name, header_value) = create_auth_header(&token);

        self.api_client
            .delete(&format!("{}/irrigation/schedule/{}", &self.address, id))
            .header(header_name, header_value)
            .send()
            .await
            .unwrap()
    }

    pub async fn get_email_verification(&self, token: String) -> reqwest::Response {
        self.api_client
            .get(&format!(
                "{}/auth/verify_email?token={}",
                &self.address, token
            ))
            .send()
            .await
            .unwrap()
    }

    pub async fn get_info(&self, token: String) -> reqwest::Response {
        let (header_name, header_value) = create_auth_header(&token);

        self.api_client
            .get(&format!("{}/info", &self.address))
            .header(header_name, header_value)
            .send()
            .await
            .unwrap()
    }

    pub async fn get_irrigation_events(&self, token: String) -> reqwest::Response {
        let (header_name, header_value) = create_auth_header(&token);

        self.api_client
            .get(&format!("{}/irrigation/event", &self.address))
            .header(header_name, header_value)
            .send()
            .await
            .unwrap()
    }

    pub async fn get_irrigation_schedule(&self, token: String, id: i32) -> reqwest::Response {
        let (header_name, header_value) = create_auth_header(&token);

        self.api_client
            .get(&format!("{}/irrigation/schedule/{}", &self.address, id))
            .header(header_name, header_value)
            .send()
            .await
            .unwrap()
    }

    pub async fn get_irrigation_schedules(&self, token: String) -> reqwest::Response {
        let (header_name, header_value) = create_auth_header(&token);

        self.api_client
            .get(&format!("{}/irrigation/schedule", &self.address))
            .header(header_name, header_value)
            .send()
            .await
            .unwrap()
    }

    pub async fn get_sump_event(&self, token: String) -> reqwest::Response {
        let (header_name, header_value) = create_auth_header(&token);

        self.api_client
            .get(&format!("{}/sump_event", &self.address))
            .header(header_name, header_value)
            .send()
            .await
            .unwrap()
    }

    pub async fn patch_irrigation_schedule(
        &self,
        token: String,
        id: i32,
        body: Value,
    ) -> reqwest::Response {
        let (header_name, header_value) = create_auth_header(&token);

        self.api_client
            .patch(&format!("{}/irrigation/schedule/{}", &self.address, id))
            .header(header_name, header_value)
            .json(&body)
            .send()
            .await
            .unwrap()
    }

    pub async fn post_heater_off(&self, token: String) -> reqwest::Response {
        let (header_name, header_value) = create_auth_header(&token);

        self.api_client
            .post(&format!("{}/heater", &self.address))
            .header(header_name, header_value)
            .json(&json!({"switch": "off"}))
            .send()
            .await
            .unwrap()
    }

    pub async fn post_heater_on(&self, token: String) -> reqwest::Response {
        let (header_name, header_value) = create_auth_header(&token);

        self.api_client
            .post(&format!("{}/heater", &self.address))
            .header(header_name, header_value)
            .json(&json!({"switch": "on"}))
            .send()
            .await
            .unwrap()
    }

    pub async fn post_irrigation_schedule(&self, token: String, body: Value) -> reqwest::Response {
        let (header_name, header_value) = create_auth_header(&token);

        self.api_client
            .post(&format!("{}/irrigation/schedule", &self.address))
            .header(header_name, header_value)
            .json(&body)
            .send()
            .await
            .unwrap()
    }

    pub async fn post_login<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.api_client
            .post(&format!("{}/auth/login", &self.address))
            .json(body)
            .send()
            .await
            .unwrap()
    }

    pub async fn post_password_reset<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.api_client
            .post(&format!("{}/auth/reset_password?token", &self.address))
            .json(body)
            .send()
            .await
            .unwrap()
    }

    pub async fn post_pool_pump(&self, token: String, body: Value) -> reqwest::Response {
        let (header_name, header_value) = create_auth_header(&token);
        self.api_client
            .post(&format!("{}/pool_pump", &self.address))
            .header(header_name, header_value)
            .json(&body)
            .send()
            .await
            .unwrap()
    }

    pub async fn post_request_password_reset<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.api_client
            .post(&format!("{}/auth/request_password_reset", &self.address))
            .json(body)
            .send()
            .await
            .unwrap()
    }

    pub async fn post_signup<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        let result = self
            .api_client
            .post(&format!("{}/auth/signup", &self.address))
            .json(body)
            .send()
            .await
            .unwrap();

        result
    }
}

pub async fn spawn_app() -> TestApp {
    spawn_app_with_gpio(get_gpio().await.expect("Couldn't get mock GPIO")).await
}

pub async fn spawn_app_with_gpio<G>(gpio: &G) -> TestApp
where
    G: Gpio,
{
    //initialize_db_template().await;

    let email_server = MockServer::start().await;
    let mut settings = Settings::new();
    settings.database_path = "".into();
    settings.server.port = 0;
    settings.mailer.server_url = email_server.uri();

    let (test_repo, temp_dir) = migrated_pathbuf().await;

    let repo = repository::implementation(Some(test_repo.to_str().unwrap().to_string()))
        .await
        .expect("Could not create repository.");

    let repo2 = repository::implementation(Some(test_repo.to_str().unwrap().to_string()))
        .await
        .expect("Could not create repository.");

    let application = Application::build(settings, gpio, repo);
    let port = application.port();

    let _ = tokio::spawn(application.run_until_stopped());

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let test_app = TestApp {
        address: format!("http://localhost:{}", port),
        port,
        repo: repo2,
        email_server,
        repo_temp_dir: temp_dir,
        api_client: client,
    };

    test_app
}

pub fn spawn_test_gpio() -> MockGpio {
    let mut gpio = MockGpio::new();

    expect_output_pin(&mut gpio, 1);
    expect_output_pin(&mut gpio, 7);
    expect_output_pin(&mut gpio, 8);
    expect_output_pin(&mut gpio, 14);
    expect_output_pin(&mut gpio, 15);
    expect_output_pin(&mut gpio, 18);
    expect_output_pin(&mut gpio, 22);
    expect_output_pin(&mut gpio, 23);
    expect_output_pin(&mut gpio, 25);
    expect_output_pin(&mut gpio, 26);
    expect_output_pin(&mut gpio, 32);

    expect_input_pin(&mut gpio, 17);
    expect_input_pin(&mut gpio, 24);
    expect_input_pin(&mut gpio, 27);

    gpio
}

fn expect_input_pin(gpio: &mut MockGpio, pin: u8) {
    gpio.expect_get().with(eq(pin)).returning(move |_| {
        let mut pin = MockPin::new();

        pin.expect_into_input_pullup().returning(|| {
            let mut input_pin = MockInputPin::new();

            input_pin
                .expect_set_async_interrupt()
                .returning(|_, _, _, _, _| Ok(()));

            input_pin.expect_read().returning(|| Level::Low);

            Box::new(input_pin)
        });

        Ok(Box::new(pin))
    });
}

fn expect_output_pin(gpio: &mut MockGpio, pin: u8) {
    gpio.expect_get().with(eq(pin)).returning(move |_| {
        let mut pin = MockPin::new();

        pin.expect_into_output_low().returning(|| {
            let mut output_pin = MockOutputPin::new();
            output_pin.expect_set_low().returning(|| ());
            Box::new(output_pin)
        });

        Ok(Box::new(pin))
    });
}
