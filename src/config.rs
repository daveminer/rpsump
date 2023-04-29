use anyhow::{anyhow, Error};
use dotenv::dotenv;
use serde::Deserialize;
use std::env;

#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub console: ConsoleConfig,
    pub database_url: String,
    pub jwt_secret: String,
    pub mailer_auth_token: String,
    pub sump: SumpConfig,
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

impl Settings {
    pub fn new() -> Result<Self, Error> {
        dotenv().ok();

        let database_url = Self::load_system_env("DATABASE_URL");
        let jwt_secret = Self::load_system_env("JWT_SECRET");
        let mailer_auth_token = Self::load_system_env("MAILER_AUTH_TOKEN");

        Ok(Settings {
            console: ConsoleConfig {
                report_freq_secs: env::var("REPORT_FREQ_SECS")
                    .unwrap_or_else(|_| "60".to_string())
                    .parse()
                    .map_err(|_| anyhow!("failed to parse report frequency"))?,
            },
            database_url,
            jwt_secret,
            mailer_auth_token,
            sump: Self::sump_config()?,
        })
    }

    fn sump_config() -> Result<SumpConfig, Error> {
        let high_sensor_pin: u8 = Self::load_system_env("JWT_SECRET").parse()?;
        let low_sensor_pin: u8 = Self::load_system_env("SUMP_LOW_SENSOR_PIN").parse()?;
        let pump_control_pin: u8 = Self::load_system_env("SUMP_PUMP_CONTROL_PIN").parse()?;
        let pump_shutoff_delay: u64 = Self::load_system_env("SUMP_PUMP_SHUTOFF_DELAY").parse()?;

        if pump_shutoff_delay >= 5 {
            panic!("SUMP_PUMP_SHUTOFF_DELAY must be 5 seconds or less.");
        }

        Ok(SumpConfig {
            high_sensor_pin,
            low_sensor_pin,
            pump_control_pin,
            pump_shutoff_delay,
        })
    }

    fn load_system_env(env: &str) -> String {
        env::var(env).expect(&format!("{} environment variable not found", env))
    }
}
