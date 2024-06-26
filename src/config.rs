use dotenv::dotenv;
use serde::Deserialize;
use std::env;

#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub console: ConsoleConfig,
    pub database_path: String,
    pub hydro: HydroConfig,
    pub jwt_secret: String,
    pub mailer: MailerConfig,
    pub server: ServerConfig,
    pub telemetry: TelemetryConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConsoleConfig {
    pub report_freq_secs: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct HeaterConfig {
    pub control_pin: u8,
}

#[derive(Clone, Debug, Deserialize)]
pub struct HydroConfig {
    pub irrigation: IrrigationConfig,
    pub heater: HeaterConfig,
    pub pool_pump: PoolPumpConfig,
    pub sump: SumpConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub struct IrrigationConfig {
    pub enabled: bool,
    pub low_sensor_pin: u8,
    pub max_seconds_runtime: u8,
    pub process_frequency_sec: u64,
    pub pump_control_pin: u8,
    pub valve_1_control_pin: u8,
    pub valve_2_control_pin: u8,
    pub valve_3_control_pin: u8,
    pub valve_4_control_pin: u8,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MailerConfig {
    pub auth_token: String,
    pub error_contact: String,
    pub server_url: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PoolPumpConfig {
    pub low_pin: u8,
    pub med_pin: u8,
    pub high_pin: u8,
    pub max_pin: u8,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ServerConfig {
    pub allow_localhost_cors: bool,
    pub host: String,
    pub port: u16,
    pub public_host: String,
    pub token_duration_days: u8,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SumpConfig {
    pub enabled: bool,
    pub high_sensor_pin: u8,
    pub low_sensor_pin: u8,
    pub pump_control_pin: u8,
    pub pump_shutoff_delay: u64,
    pub pump_max_runtime: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TelemetryConfig {
    pub api_key: String,
    pub receiver_url: String,
}

impl Settings {
    pub fn new() -> Self {
        set_application_environment();

        let database_path = load_system_var("DATABASE_PATH");
        let jwt_secret = load_system_var("JWT_SECRET");
        let server_host = load_system_var("SERVER_HOST");
        let server_port: u16 = load_system_var("SERVER_PORT")
            .parse()
            .expect("SERVER_PORT must be a 16-bit unsigned integer.");

        Settings {
            console: ConsoleConfig {
                report_freq_secs: env::var("CONSOLE_REPORT_FREQ_SECS")
                    .unwrap_or_else(|_| "5".to_string())
                    .parse()
                    .expect("CONSOLE_REPORT_FREQ_SECS must be a number."),
            },
            database_path,
            hydro: HydroConfig {
                irrigation: Self::irrigation_config().expect("Could not load irrigation config."),
                heater: HeaterConfig {
                    control_pin: load_system_var("HEATER_CONTROL_PIN")
                        .parse()
                        .expect("HEATER_CONTROL_PIN must be a number."),
                },
                pool_pump: PoolPumpConfig {
                    low_pin: load_system_var("POOL_PUMP_LOW_PIN")
                        .parse()
                        .expect("POOL_PUMP_LOW_PIN must be a number."),
                    med_pin: load_system_var("POOL_PUMP_MED_PIN")
                        .parse()
                        .expect("POOL_PUMP_MED_PIN must be a number."),
                    high_pin: load_system_var("POOL_PUMP_HIGH_PIN")
                        .parse()
                        .expect("POOL_PUMP_HIGH_PIN must be a number."),
                    max_pin: load_system_var("POOL_PUMP_MAX_PIN")
                        .parse()
                        .expect("POOL_PUMP_MAX_PIN must be a number."),
                },
                sump: Self::sump_config().expect("Could not load sump config."),
            },
            jwt_secret,
            mailer: MailerConfig {
                auth_token: load_system_var("MAILER_AUTH_TOKEN"),
                error_contact: load_system_var("MAILER_ERROR_CONTACT"),
                server_url: load_system_var("MAILER_SERVER_URL"),
            },
            server: ServerConfig {
                allow_localhost_cors: load_system_var("SERVER_ALLOW_LOCALHOST_CORS")
                    .parse()
                    .expect("SERVER_ALLOW_LOCALHOST_CORS must be a boolean."),
                host: server_host,
                port: server_port,
                public_host: load_system_var("PUBLIC_HOST"),
                token_duration_days: load_system_var("SERVER_TOKEN_DURATION_DAYS")
                    .parse()
                    .expect("TOKEN_DURATION must be a number."),
            },
            telemetry: TelemetryConfig {
                api_key: load_system_var("TELEMETRY_API_KEY"),
                receiver_url: load_system_var("TELEMETRY_RECEIVER_URL"),
            },
        }
    }

    fn irrigation_config() -> Option<IrrigationConfig> {
        let enabled: bool = load_system_var("IRRIGATION_ENABLED")
            .parse()
            .expect("IRRIGATION_ENABLED must be a boolean.");

        let low_sensor_pin: u8 = load_system_var("IRRIGATION_LOW_SENSOR_PIN")
            .parse()
            .expect("IRRIGATION_LOW_SENSOR_PIN must be a number.");
        let max_seconds_runtime: u8 = load_system_var("IRRIGATION_MAX_RUNTIME")
            .parse()
            .expect("IRRIGATION_MAX_RUNTIME must be a number");
        let process_frequency_sec: u64 = load_system_var("IRRIGATION_PROCESS_FREQ_SEC")
            .parse()
            .expect("IRRIGATION_PROCESS_FREQ_SEC must be a number.");
        let pump_control_pin: u8 = load_system_var("IRRIGATION_PUMP_CONTROL_PIN")
            .parse()
            .expect("IRRIGATION_PUMP_CONTROL_PIN must be a number.");
        let valve_1_control_pin: u8 = load_system_var("IRRIGATION_VALVE_1_CONTROL_PIN")
            .parse()
            .expect("IRRIGATION_VALVE_1_CONTROL_PIN must be a number.");
        let valve_2_control_pin: u8 = load_system_var("IRRIGATION_VALVE_2_CONTROL_PIN")
            .parse()
            .expect("IRRIGATION_VALVE_2_CONTROL_PIN must be a number.");
        let valve_3_control_pin: u8 = load_system_var("IRRIGATION_VALVE_3_CONTROL_PIN")
            .parse()
            .expect("IRRIGATION_VALVE_3_CONTROL_PIN must be a number.");
        let valve_4_control_pin: u8 = load_system_var("IRRIGATION_VALVE_4_CONTROL_PIN")
            .parse()
            .expect("IRRIGATION_VALVE_4_CONTROL_PIN must be a number.");

        Some(IrrigationConfig {
            enabled,
            low_sensor_pin,
            max_seconds_runtime,
            process_frequency_sec,
            pump_control_pin,
            valve_1_control_pin,
            valve_2_control_pin,
            valve_3_control_pin,
            valve_4_control_pin,
        })
    }

    fn sump_config() -> Option<SumpConfig> {
        let enabled: bool = load_system_var("SUMP_ENABLED")
            .parse()
            .expect("SUMP_ENABLED must be a boolean.");

        let high_sensor_pin: u8 = load_system_var("SUMP_HIGH_SENSOR_PIN")
            .parse()
            .expect("SUMP_HIGH_SENSOR_PIN must be a number.");
        let low_sensor_pin: u8 = load_system_var("SUMP_LOW_SENSOR_PIN")
            .parse()
            .expect("SUMP_LOW_SENSOR_PIN must be a number.");
        let pump_control_pin: u8 = load_system_var("SUMP_CONTROL_PIN")
            .parse()
            .expect("SUMP_CONTROL_PIN must be a number.");
        let pump_max_runtime = load_system_var("SUMP_PUMP_MAX_RUNTIME")
            .parse()
            .expect("SUMP_PUMP_MAX_RUNTIME must be a number.");
        let pump_shutoff_delay: u64 = load_system_var("SUMP_SHUTOFF_DELAY")
            .parse()
            .expect("SUMP_SHUTOFF_DELAY must be a number.");

        if pump_shutoff_delay >= 5 {
            panic!("SUMP_SHUTOFF_DELAY must be 5 seconds or less.");
        }

        Some(SumpConfig {
            enabled,
            high_sensor_pin,
            low_sensor_pin,
            pump_control_pin,
            pump_shutoff_delay,
            pump_max_runtime,
        })
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

fn load_system_var(env: &str) -> String {
    env::var(env).unwrap_or_else(|_| panic!("{} environment variable not found.", env))
}

fn set_application_environment() {
    if std::env::var("RPSUMP_TEST").is_ok() {
        dotenv::from_filename(".env.test").ok();
        return;
    }

    dotenv().ok();
}
