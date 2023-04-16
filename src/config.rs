use anyhow::{anyhow, Error};
use dotenv::dotenv;
use serde::Deserialize;
use std::env;

#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub console: ConsoleConfig,
    database: DatabaseConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConsoleConfig {
    pub report_freq_secs: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DatabaseConfig {
    path: String,
    filename: String,
}

impl Settings {
    pub fn new() -> Result<Self, Error> {
        dotenv().ok();

        let database_path = env::var("DATABASE_PATH")
            .map_err(|_| anyhow!("DATABASE_PATH environment variable not found"))?;
        let database_file = env::var("DATABASE_FILE")
            .map_err(|_| anyhow!("DATABASE_FILE environment variable not found"))?;

        Ok(Settings {
            console: ConsoleConfig {
                report_freq_secs: env::var("REPORT_FREQ_SECS")
                    .unwrap_or_else(|_| "60".to_string())
                    .parse()
                    .map_err(|_| anyhow!("failed to parse report frequency"))?,
            },
            database: DatabaseConfig {
                path: database_path,
                filename: database_file,
            },
        })
    }

    pub fn database(self) -> String {
        let file = self.database.filename;
        let mut path = self.database.path;

        if !file.starts_with("/") && !path.ends_with("/") {
            path = format!("{}/", path);
        }

        format!("{}{}", path, file)
    }
}
