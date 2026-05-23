use anyhow::Error;
use tokio::{
    runtime::Handle,
    sync::mpsc::{Receiver, Sender},
};

use crate::{
    config::HydroConfig,
    hydro::{
        garden::Garden, gpio::Gpio, heater::Heater, pool_pump::PoolPump, sump::Sump,
        weather::WeatherClient,
    },
    repository::Repo,
};

use self::signal::Signal;

pub mod control;
pub mod debounce;
pub mod garden;
pub mod gpio;
pub mod heater;
pub mod pool_pump;
pub mod sensor;
pub mod signal;
mod sump;
pub mod weather;

// Re-exports so siblings can reference `crate::hydro::Control` / `crate::hydro::Level`
// (matches the convention from prior to the garden refactor).
pub use control::Control;
pub use gpio::Level;

pub struct Hydro {
    pub repo: Repo,
    pub heater: Heater,
    pub pool_pump: PoolPump,
    pub handle: Handle,
    pub sump: Sump,
    pub garden: Garden,
}

impl Hydro {
    pub fn new(
        config: &HydroConfig,
        handle: Handle,
        gpio: &dyn Gpio,
        repo: Repo,
    ) -> Result<Self, Error> {
        let mpsc: (Sender<Signal>, Receiver<Signal>) = tokio::sync::mpsc::channel(32);
        let tx = mpsc.0;

        let heater = Heater::new(&config.heater, gpio)?;
        let pool_pump = PoolPump::new(&config.pool_pump, gpio)?;

        let sump = Sump::new(&config.sump, &tx, handle.clone(), gpio)?;
        let garden = Garden::new(&config.garden, gpio)?;
        let weather = WeatherClient::new(config.weather.clone());

        garden::schedule::start(
            repo,
            garden.clone(),
            weather,
            config.garden.process_frequency_sec,
        );

        signal::listen(
            mpsc.1,
            handle.clone(),
            sump.pump.pin.clone(),
            config.sump.pump_shutoff_delay,
            config.sump.pump_max_runtime,
        );

        Ok(Self {
            garden,
            heater,
            pool_pump,
            repo,
            handle,
            sump,
        })
    }
}
