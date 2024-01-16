use anyhow::Error;
use futures::try_join;

use crate::{
    config::PoolPumpConfig,
    hydro::{control::Output, gpio::Gpio, Control},
};

#[derive(Clone)]
pub struct PoolPump {
    low: Control,
    med: Control,
    high: Control,
    max: Control,
    current: PoolPumpSpeed,
}

#[derive(Clone)]
enum PoolPumpSpeed {
    Off,
    Low,
    Med,
    High,
    Max,
}

impl PoolPump {
    pub fn new<G>(config: &PoolPumpConfig, gpio: &G) -> Result<Self, Error>
    where
        G: Gpio,
    {
        let low = Control::new("low speed".into(), config.low_pin, gpio)?;
        let med = Control::new("medium speed".into(), config.med_pin, gpio)?;
        let high = Control::new("high speed".into(), config.high_pin, gpio)?;
        let max = Control::new("max speed".into(), config.max_pin, gpio)?;

        Ok(Self {
            low,
            med,
            high,
            max,
            current: PoolPumpSpeed::Off,
        })
    }

    async fn off(&mut self) -> Result<(), Error> {
        let low = turn_off(&mut self.low);
        let med = turn_off(&mut self.med);
        let high = turn_off(&mut self.high);
        let max = turn_off(&mut self.max);

        try_join!(low, med, high, max).map(|_| ())
    }

    async fn on(mut self, speed: PoolPumpSpeed) -> Result<(), Error> {
        match speed {
            PoolPumpSpeed::Off => self.off().await,
            PoolPumpSpeed::Low => self.low.on().await,
            PoolPumpSpeed::Med => self.med.on().await,
            PoolPumpSpeed::High => self.high.on().await,
            PoolPumpSpeed::Max => self.max.on().await,
        }
    }
}

async fn turn_off(speed_pin: &mut Control) -> Result<(), Error> {
    if speed_pin.is_on() {
        return speed_pin.off().await;
    }
    Ok(())
}
