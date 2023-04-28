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

        let database_url = env::var("DATABASE_URL")
            .map_err(|_| anyhow!("DATABASE_URL environment variable not found"))?;

        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_| anyhow!("JWT_SECRET environment variable not found"))?;

        let mailer_auth_token = env::var("MAILER_AUTH_TOKEN")
            .map_err(|_| anyhow!("MAILER_AUTH_TOKEN environment variable not found"))?;

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
        let high_sensor_pin = env::var("SUMP_HIGH_SENSOR_PIN")
            .map_err(|_| anyhow!("SUMP_HIGH_SENSOR_PIN environment variable not found"))?
            .parse::<u8>()?;

        let low_sensor_pin = env::var("SUMP_LOW_SENSOR_PIN")
            .map_err(|_| Self::not_found_err("SUMP_LOW_SENSOR_PIN"))?
            .parse::<u8>()?;

        let pump_control_pin = env::var("SUMP_PUMP_CONTROL_PIN")
            .map_err(|_| anyhow!("SUMP_PUMP_CONTROL_PIN environment variable not found"))?
            .parse::<u8>()?;

        let pump_shutoff_delay = env::var("SUMP_PUMP_SHUTOFF_DELAY")
            .map_err(|_| anyhow!("SUMP_PUMP_SHUTOFF_DELAY environment variable not found"))?
            .parse::<u64>()?;

        if pump_shutoff_delay >= 5 {
            return Err(anyhow!(
                "SUMP_PUMP_SHUTOFF_DELAY must be 5 seconds or less."
            ));
        }

        Ok(SumpConfig {
            high_sensor_pin,
            low_sensor_pin,
            pump_control_pin,
            pump_shutoff_delay,
        })
    }

    fn not_found_err(env_var: &str) -> Error {
        anyhow!("SUMP_HIGH_SENSOR_PIN environment variable not found")
    }
}
