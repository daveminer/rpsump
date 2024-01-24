use anyhow::Error;
use std::sync::{Arc, Mutex};

use crate::{
    config::HydroConfig,
    hydro::{
        control::Control,
        gpio::{Gpio, Level},
        heater::Heater,
        irrigator::Irrigator,
        message::Message,
        pool_pump::PoolPump,
        sump::Sump,
    },
    repository::Repo,
};

pub mod control;
pub mod debounce;
pub mod gpio;
pub mod heater;
mod irrigator;
mod message;
pub mod pool_pump;
pub mod schedule;
pub mod sensor;
mod sump;

#[derive(Clone)]
pub struct Hydro {
    pub repo: Repo,
    pub heater: Heater,
    pub pool_pump: PoolPump,
    pub sump: Sump,
    pub irrigator: Irrigator,
}

impl Hydro {
    pub fn new<G>(config: &HydroConfig, gpio: &G, repo: Repo) -> Result<Self, Error>
    where
        G: Gpio,
    {
        let mpsc = Message::init();
        let tx = mpsc.tx;

        let heater = Heater::new(&config.heater, gpio)?;
        let pool_pump = PoolPump::new(&config.pool_pump, gpio)?;

        let rt = actix_web::rt::Runtime::new().expect("Could not create runtime");
        let rt_tokio = rt.tokio_runtime();
        let handle = rt_tokio.handle();
        // let rt = Arc::from(Mutex::new(
        //     actix_web::rt::Runtime::new().expect("Could not create runtime"),
        // ));

        let sump = Sump::new(&config.sump, &tx, handle.clone(), gpio)?;
        let irrigator = Irrigator::new(&config.irrigation, &tx, handle.clone(), gpio)?;

        Ok(Self {
            irrigator,
            heater,
            pool_pump,
            repo,
            sump,
        })
    }
}
