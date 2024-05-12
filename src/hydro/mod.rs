use anyhow::Error;
use tokio::{
    runtime::Handle,
    sync::mpsc::{Receiver, Sender},
};

use crate::{
    config::HydroConfig,
    hydro::{
        control::Control,
        gpio::{Gpio, Level},
        heater::Heater,
        irrigator::Irrigator,
        pool_pump::PoolPump,
        sump::Sump,
    },
    repository::Repo,
};

use self::signal::Signal;

pub mod control;
pub mod debounce;
pub mod gpio;
pub mod heater;
pub mod irrigator;
pub mod pool_pump;
pub mod schedule;
pub mod sensor;
pub mod signal;
mod sump;

pub struct Hydro {
    pub repo: Repo,
    pub heater: Heater,
    pub pool_pump: PoolPump,
    pub handle: Handle,
    pub sump: Sump,
    pub irrigator: Irrigator,
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
        let irrigator = Irrigator::new(&config.irrigation, &tx, handle.clone(), gpio)?;

        schedule::start(
            repo,
            irrigator.clone(),
            config.irrigation.process_frequency_sec,
        );

        signal::listen(
            mpsc.1,
            handle.clone(),
            irrigator.pump.pin.clone(),
            sump.pump.pin.clone(),
            config.sump.pump_shutoff_delay,
        );

        Ok(Self {
            irrigator,
            heater,
            pool_pump,
            repo,
            handle,
            sump,
        })
    }
}
