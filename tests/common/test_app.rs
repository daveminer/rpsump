use mockall::predicate::eq;
use once_cell::sync::{Lazy, OnceCell};
use rpsump::hydro::gpio::{Gpio, Level, MockGpio, MockInputPin, MockOutputPin, MockPin};
use serde_json::{json, Value};
use std::fs::copy;
use tempfile::TempDir;
use uuid::Uuid;
use wiremock::MockServer;

use rpsump::config::Settings;
use rpsump::database::{new_pool, DbPool};
use rpsump::startup::Application;

// TODO: move to shared location
use crate::auth::authenticated_user::create_auth_header;

pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub db_pool: DbPool,
    pub email_server: MockServer,
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
        .unwrap()
        .get()
        .unwrap();
    conn.run_pending_migrations(MIGRATIONS)
        .expect("diesel migrations");

    temp_dir
});

static GPIO: OnceCell<MockGpio> = OnceCell::new();

pub fn get_gpio() -> &'static MockGpio {
    GPIO.get_or_init(|| spawn_test_gpio())
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

pub fn spawn_test_db() -> String {
    let test_db_dir = Lazy::force(&TEST_DB_TEMPLATE);
    let test_db_path = test_db_dir.path().to_str().unwrap().to_string();
    let template_db = format!("{}/{}", test_db_path, DB_TEMPLATE_FILE);

    let db_instance_file = format!("{}.db", Uuid::new_v4().to_string());
    let db_instance = format!("{}/{}", test_db_path, db_instance_file);
    let pool = new_pool(&db_instance);
    let _conn = pool.unwrap().get().unwrap();

    // copy
    copy(&template_db, &db_instance).unwrap();

    db_instance
}

pub async fn spawn_app() -> TestApp {
    spawn_app_with_gpio(get_gpio()).await
}

pub async fn spawn_app_with_gpio<G>(gpio: &G) -> TestApp
where
    G: Gpio,
{
    let email_server = MockServer::start().await;

    let mut settings = Settings::new();
    settings.database_url = spawn_test_db();
    settings.server.port = 0;

    settings.mailer.server_url = email_server.uri();

    let db_pool = new_pool(&spawn_test_db()).unwrap();

    let application = Application::build(settings, &db_pool, gpio);
    let port = application.port();

    let _ = tokio::spawn(application.run_until_stopped());

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let test_app = TestApp {
        address: format!("http://localhost:{}", port),
        port,
        db_pool,
        email_server,
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
                .returning(|_, _| Ok(()));

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
