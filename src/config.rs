use anyhow::{anyhow, Error};
use dotenv::dotenv;
use serde::Deserialize;
use std::env;

#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub console: ConsoleConfig,
    pub database_url: String,
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
}

impl Settings {
    pub fn new() -> Result<Self, Error> {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .map_err(|_| anyhow!("DATABASE_URL environment variable not found"))?;

        let high_sensor_pin = env::var("DATABASE_URL")
            .map_err(|_| anyhow!("DATABASE_URL environment variable not found"))?
            .parse::<u8>()?;

        let low_sensor_pin = env::var("DATABASE_URL")
            .map_err(|_| anyhow!("DATABASE_URL environment variable not found"))?
            .parse::<u8>()?;

        let pump_control_pin = env::var("DATABASE_URL")
            .map_err(|_| anyhow!("DATABASE_URL environment variable not found"))?
            .parse::<u8>()?;

        Ok(Settings {
            console: ConsoleConfig {
                report_freq_secs: env::var("REPORT_FREQ_SECS")
                    .unwrap_or_else(|_| "60".to_string())
                    .parse()
                    .map_err(|_| anyhow!("failed to parse report frequency"))?,
            },
            database_url: database_url,
            sump: SumpConfig {
                high_sensor_pin,
                low_sensor_pin,
                pump_control_pin,
            },
        })
    }
}
