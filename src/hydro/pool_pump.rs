use anyhow::Error;
use futures::try_join;
use serde::Deserialize;

use crate::{
    config::PoolPumpConfig,
    hydro::{control::Output, gpio::Gpio, Control},
};

#[derive(Clone)]
pub struct PoolPump {
    pub low: Control,
    pub med: Control,
    pub high: Control,
    pub max: Control,
    pub current: PoolPumpSpeed,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PoolPumpSpeed {
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

    pub async fn off(&mut self) -> Result<(), Error> {
        let low = turn_off(&mut self.low);
        let med = turn_off(&mut self.med);
        let high = turn_off(&mut self.high);
        let max = turn_off(&mut self.max);

        try_join!(low, med, high, max).map(|_| ())
    }

    /// Sets the new speed on the pump. This pump accepts four 5v inputs, and
    /// will set the speed according to the highest speed input that is active.
    /// For this reason, the new speed input is raised before lowering the pin
    /// for the old speed as to avoid an extra shift to the "off" state
    /// between speed changes.
    pub async fn on(&mut self, speed: PoolPumpSpeed) -> Result<(), Error> {
        match speed {
            PoolPumpSpeed::Off => self.off().await?,
            PoolPumpSpeed::Low => self.low.on().await?,
            PoolPumpSpeed::Med => self.med.on().await?,
            PoolPumpSpeed::High => self.high.on().await?,
            PoolPumpSpeed::Max => self.max.on().await?,
        };

        match self.current {
            PoolPumpSpeed::Off => (),
            PoolPumpSpeed::Low => self.low.off().await?,
            PoolPumpSpeed::Med => self.med.off().await?,
            PoolPumpSpeed::High => self.high.off().await?,
            PoolPumpSpeed::Max => self.max.off().await?,
        };

        self.current = speed;

        Ok(())
    }
}

async fn turn_off(speed_pin: &mut Control) -> Result<(), Error> {
    if speed_pin.is_on() {
        return speed_pin.off().await;
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::test_fixtures::gpio::mock_gpio_get;

    use super::*;

    #[tokio::test]
    async fn test_pool_pump_new() {
        let config = PoolPumpConfig {
            low_pin: 1,
            med_pin: 2,
            high_pin: 3,
            max_pin: 4,
        };
        let mock_gpio = mock_gpio_get(vec![1, 2, 3, 4]);

        let pool_pump = PoolPump::new(&config, &mock_gpio).unwrap();

        assert_eq!(pool_pump.current, PoolPumpSpeed::Off);
    }

    #[tokio::test]
    async fn test_pool_pump_off() {
        let config = PoolPumpConfig {
            low_pin: 1,
            med_pin: 2,
            high_pin: 3,
            max_pin: 4,
        };
        let mock_gpio = mock_gpio_get(vec![1, 2, 3, 4]);

        let mut pool_pump = PoolPump::new(&config, &mock_gpio).unwrap();
        pool_pump.off().await.unwrap();

        assert_eq!(pool_pump.current, PoolPumpSpeed::Off);
    }

    #[tokio::test]
    async fn test_pool_pump_on() {
        let config = PoolPumpConfig {
            low_pin: 1,
            med_pin: 2,
            high_pin: 3,
            max_pin: 4,
        };
        let mock_gpio = mock_gpio_get(vec![1, 2, 3, 4]);

        let mut pool_pump = PoolPump::new(&config, &mock_gpio).unwrap();
        let _ok = &pool_pump.on(PoolPumpSpeed::Med).await.unwrap();

        assert_eq!(pool_pump.current, PoolPumpSpeed::Med);
    }
}
