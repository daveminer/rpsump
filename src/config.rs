use dotenv::dotenv;
use serde::Deserialize;
use std::env;

#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub auth_attempts_allowed: i64,
    pub console: ConsoleConfig,
    pub database_url: String,
    pub jwt_secret: String,
    pub mailer: MailerConfig,
    pub rate_limiter: ThrottleConfig,
    pub server: ServerConfig,
    pub sump: Option<SumpConfig>,
    pub telemetry: TelemetryConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConsoleConfig {
    pub report_freq_secs: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MailerConfig {
    pub auth_token: String,
    pub server_url: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SumpConfig {
    pub high_sensor_pin: u8,
    pub low_sensor_pin: u8,
    pub pump_control_pin: u8,
    pub pump_shutoff_delay: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TelemetryConfig {
    pub api_key: String,
    pub receiver_url: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ThrottleConfig {
    pub per_second: u64,
    pub burst_size: u32,
}

impl Settings {
    pub fn new() -> Self {
        set_application_environment();

        let database_url = load_system_var("DATABASE_URL");
        let jwt_secret = load_system_var("JWT_SECRET");
        let server_host = load_system_var("SERVER_HOST");
        let server_port: u16 = load_system_var("SERVER_PORT")
            .parse()
            .expect("SERVER_PORT must be a 16-bit unsigned integer.");

        Settings {
            auth_attempts_allowed: env::var("AUTH_ATTEMPTS_ALLOWED")
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .expect("AUTH_ATTEMPTS_ALLOWED must be a number."),
            console: ConsoleConfig {
                report_freq_secs: env::var("CONSOLE_REPORT_FREQ_SECS")
                    .unwrap_or_else(|_| "5".to_string())
                    .parse()
                    .expect("CONSOLE_REPORT_FREQ_SECS must be a number."),
            },
            database_url,
            jwt_secret,
            mailer: MailerConfig {
                auth_token: load_system_var("MAILER_AUTH_TOKEN"),
                server_url: load_system_var("MAILER_SERVER_URL"),
            },
            rate_limiter: ThrottleConfig {
                per_second: env::var("RATE_LIMIT_PER_SECOND")
                    .unwrap_or_else(|_| "2".to_string())
                    .parse()
                    .expect("PER_SECOND must be a number."),
                burst_size: env::var("RATE_LIMIT_BURST_SIZE")
                    .unwrap_or_else(|_| "5".to_string())
                    .parse()
                    .expect("BURST_SIZE must be a number."),
            },
            server: ServerConfig {
                host: server_host,
                port: server_port,
            },
            sump: Self::sump_config(),
            telemetry: TelemetryConfig {
                api_key: load_system_var("TELEMETRY_API_KEY"),
                receiver_url: load_system_var("TELEMETRY_RECEIVER_URL"),
            },
        }
    }

    fn sump_config() -> Option<SumpConfig> {
        if !load_system_var("SUMP_ENABLED")
            .parse::<bool>()
            .expect("SUMP_ENABLED must be a boolean.")
        {
            return None;
        }

        let high_sensor_pin: u8 = load_system_var("SUMP_HIGH_SENSOR_PIN")
            .parse()
            .expect("SUMP_HIGH_SENSOR_PIN must be a number.");
        let low_sensor_pin: u8 = load_system_var("SUMP_LOW_SENSOR_PIN")
            .parse()
            .expect("SUMP_LOW_SENSOR_PIN must be a number.");
        let pump_control_pin: u8 = load_system_var("SUMP_CONTROL_PIN")
            .parse()
            .expect("SUMP_CONTROL_PIN must be a number.");
        let pump_shutoff_delay: u64 = load_system_var("SUMP_SHUTOFF_DELAY")
            .parse()
            .expect("SUMP_SHUTOFF_DELAY must be a number.");

        if pump_shutoff_delay >= 5 {
            panic!("SUMP_SHUTOFF_DELAY must be 5 seconds or less.");
        }

        Some(SumpConfig {
            high_sensor_pin,
            low_sensor_pin,
            pump_control_pin,
            pump_shutoff_delay,
        })
    }
}

fn load_system_var(env: &str) -> String {
    env::var(env).expect(&format!("{} environment variable not found.", env))
}

fn set_application_environment() {
    let environment = env::var("RPSUMP_ENVIRONMENT").unwrap_or_else(|_e| "development".to_string());

    if environment == "test" {
        dotenv::from_filename(".env.test").ok();
        return;
    }

    dotenv().ok();
}
