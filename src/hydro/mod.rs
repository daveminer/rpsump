use anyhow::Error;
use std::sync::{Arc, Mutex};

use crate::{
    config::HydroConfig,
    database::DbPool,
    hydro::{
        control::Control,
        gpio::{Gpio, Level},
        heater::Heater,
        irrigator::Irrigator,
        message::Message,
        pool_pump::PoolPump,
        sump::Sump,
    },
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
    pub db_pool: Box<dyn DbPool>,
    pub heater: Heater,
    pub pool_pump: PoolPump,
    pub sump: Sump,
    pub irrigator: Irrigator,
}

impl Clone for Box<dyn DbPool> {
    fn clone(&self) -> Box<dyn DbPool> {
        self.clone_box()
    }
}

impl Hydro {
    pub fn new<D, G>(db: D, config: &HydroConfig, gpio: &G) -> Result<Self, Error>
    where
        D: DbPool,
        G: Gpio,
    {
        let mpsc = Message::init();
        let tx = mpsc.tx;

        let heater = Heater::new(&config.heater, gpio)?;
        let pool_pump = PoolPump::new(&config.pool_pump, gpio)?;

        let rt = Arc::from(Mutex::new(
            actix_web::rt::Runtime::new().expect("Could not create runtime"),
        ));
        let sump = Sump::new(&config.sump, &tx, rt, gpio)?;
        let irrigator = Irrigator::new(&config.irrigation, &tx, rt, gpio)?;

        Ok(Self {
            db_pool: Box::new(db),
            irrigator,
            heater,
            pool_pump,
            sump,
        })
    }
}
