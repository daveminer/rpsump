use anyhow::Error;
use tokio::runtime::Handle;

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

pub mod control;
pub mod debounce;
pub mod gpio;
pub mod heater;
mod irrigator;
pub mod pool_pump;
pub mod schedule;
pub mod sensor;
pub mod signal;
mod sump;

#[derive(Clone)]
pub struct Hydro {
    pub repo: Repo,
    pub heater: Heater,
    pub pool_pump: PoolPump,
    pub handle: Handle,
    pub sump: Sump,
    pub irrigator: Irrigator,
}

impl Hydro {
    pub fn new<G>(
        config: &HydroConfig,
        handle: Handle,
        gpio: Box<dyn Gpio>,
        repo: Repo,
    ) -> Result<Self, Error>
    where
        G: Gpio,
    {
        let mpsc = tokio::sync::mpsc::channel(32);
        let tx = mpsc.0;

        let heater = Heater::new(&config.heater, &gpio)?;
        let pool_pump = PoolPump::new(&config.pool_pump, &gpio)?;

        let sump = Sump::new(&config.sump, &tx, &gpio)?;
        let irrigator = Irrigator::new(&config.irrigation, &tx, gpio)?;

        signal::listen(
            mpsc.1,
            handle.clone(),
            irrigator.clone(),
            None,
            sump.clone(),
            None,
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

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::Hydro;
    use crate::{
        config::HydroConfig,
        repository::MockRepository,
        test_fixtures::{gpio::mock_gpio_get, hydro::hydro_config},
    };

    #[rstest]
    #[tokio::test]
    async fn test_new(#[from(hydro_config)] hydro_config: HydroConfig) {
        let mock_gpio = mock_gpio_get(vec![1, 2, 3, 4, 5, 6, 7, 8, 10, 12, 13, 14, 15, 16]);
        let mock_repo = MockRepository::new();
        let handle = tokio::runtime::Handle::current();

        let mock_repo_borrow = Box::leak(Box::new(mock_repo));
        let result = Hydro::new(&hydro_config, handle, &mock_gpio, mock_repo_borrow);

        assert!(result.is_ok());
    }
}
