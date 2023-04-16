use anyhow::{anyhow, Error};
use config::{Config, File};
use serde::Deserialize;

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
        let s = Config::builder()
            // Start off by merging in the "default" configuration file
            .add_source(File::with_name("./config/default.toml"))
            // Add in a local configuration file
            // This file shouldn't be checked in to git
            .add_source(File::with_name("./config/dev.secret.toml").required(false))
            .build()?;

        // Now that we're done, let's access our configuration
        println!("debug: {:?}", s.get_string("console.report_freq_secs"));
        println!("database: {:?}", s.get::<String>("database.path"));

        match s.try_deserialize() {
            Ok(settings) => Ok(settings),
            Err(e) => Err(anyhow!(e)),
        }
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
