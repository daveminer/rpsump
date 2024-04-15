use crate::config::{HeaterConfig, HydroConfig, IrrigationConfig, PoolPumpConfig, SumpConfig};

use rstest::fixture;

#[fixture]
pub fn hydro_config() -> HydroConfig {
    HydroConfig {
        heater: HeaterConfig { control_pin: 1 },
        pool_pump: PoolPumpConfig {
            low_pin: 2,
            med_pin: 3,
            high_pin: 4,
            max_pin: 5,
        },
        sump: SumpConfig {
            enabled: true,
            high_sensor_pin: 6,
            low_sensor_pin: 7,
            pump_control_pin: 8,
            pump_shutoff_delay: 9,
        },
        irrigation: IrrigationConfig {
            enabled: true,
            low_sensor_pin: 10,
            max_seconds_runtime: 11,
            process_frequency_sec: 1000,
            pump_control_pin: 12,
            valve_1_control_pin: 13,
            valve_2_control_pin: 14,
            valve_3_control_pin: 15,
            valve_4_control_pin: 16,
        },
    }
}
