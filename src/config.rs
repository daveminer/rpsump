use anyhow::Error;
use dotenv::dotenv;
use serde::Deserialize;
use std::env;

#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub auth_attempts_allowed: i64,
    pub console: ConsoleConfig,
    pub database_url: String,
    pub jwt_secret: String,
    pub mailer_auth_token: String,
    pub rate_limiter: ThrottleConfig,
    pub sump: Option<SumpConfig>,
    pub telemetry: TelemetryConfig,
    pub user_activation_required: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConsoleConfig {
    pub report_freq_secs: u64,
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
    pub fn new() -> Result<Self, Error> {
        dotenv().ok();

        let database_url = Self::load_system_env("DATABASE_URL");
        let jwt_secret = Self::load_system_env("JWT_SECRET");
        let mailer_auth_token = Self::load_system_env("MAILER_AUTH_TOKEN");

        Ok(Settings {
            auth_attempts_allowed: env::var("AUTH_ATTEMPTS_ALLOWED")
                .unwrap_or_else(|_| "3".to_string())
                .parse()?,
            console: ConsoleConfig {
                report_freq_secs: env::var("CONSOLE_REPORT_FREQ_SECS")
                    .unwrap_or_else(|_| "5".to_string())
                    .parse()?,
            },
            database_url,
            jwt_secret,
            mailer_auth_token,
            rate_limiter: ThrottleConfig {
                per_second: env::var("RATE_LIMIT_PER_SECOND")
                    .unwrap_or_else(|_| "2".to_string())
                    .parse()?,
                burst_size: env::var("RATE_LIMIT_BURST_SIZE")
                    .unwrap_or_else(|_| "5".to_string())
                    .parse()?,
            },
            sump: Self::sump_config()?,
            telemetry: TelemetryConfig {
                api_key: Self::load_system_env("TELEMETRY_API_KEY"),
                receiver_url: Self::load_system_env("TELEMETRY_RECEIVER_URL"),
            },
            user_activation_required: env::var("USER_ACTIVATION_REQUIRED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()?,
        })
    }

    fn sump_config() -> Result<Option<SumpConfig>, Error> {
        if !Self::load_system_env("SUMP_ENABLED").parse()? {
            return Ok(None);
        }

        let high_sensor_pin: u8 = Self::load_system_env("SUMP_HIGH_SENSOR_PIN").parse()?;
        let low_sensor_pin: u8 = Self::load_system_env("SUMP_LOW_SENSOR_PIN").parse()?;
        let pump_control_pin: u8 = Self::load_system_env("SUMP_CONTROL_PIN").parse()?;
        let pump_shutoff_delay: u64 = Self::load_system_env("SUMP_SHUTOFF_DELAY").parse()?;

        if pump_shutoff_delay >= 5 {
            panic!("SUMP_SHUTOFF_DELAY must be 5 seconds or less.");
        }

        Ok(Some(SumpConfig {
            high_sensor_pin,
            low_sensor_pin,
            pump_control_pin,
            pump_shutoff_delay,
        }))
    }

    fn load_system_env(env: &str) -> String {
        env::var(env).expect(&format!("{} environment variable not found", env))
    }
}

#[test]
fn test_new_settings() {
    dotenv().ok();

    assert!(Settings::new().is_ok());
}
