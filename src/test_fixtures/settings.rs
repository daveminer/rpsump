use lazy_static::lazy_static;

use crate::config::Settings;

// Create a Settings instance to be used in tests
lazy_static! {
    pub static ref SETTINGS: Settings = {
        dotenv::from_filename(".env.test").ok();

        Settings::new()
    };
}
