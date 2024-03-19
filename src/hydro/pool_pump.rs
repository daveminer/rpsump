use anyhow::Error;
use serde::Deserialize;
use tracing::error;

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

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
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
        let mut self_clone = self.clone();

        let low_result = self_clone.low.off().await;
        if let Err(e) = low_result {
            error!("Error setting pool pump off: {}", e);
        }
        let med_result = self_clone.med.off().await;
        if let Err(e) = med_result {
            error!("Error setting pool pump off: {}", e);
        }
        let high_result = self_clone.high.off().await;
        if let Err(e) = high_result {
            error!("Error setting pool pump off: {}", e);
        }
        let max_result = self_clone.max.off().await;
        if let Err(e) = max_result {
            error!("Error setting pool pump off: {}", e);
        }

        self.current = PoolPumpSpeed::Off;

        Ok(())
    }

    /// Sets the new speed on the pump. This pump accepts four 5v inputs, and
    /// will set the speed according to the highest speed input that is active.
    /// For this reason, the new speed input is raised before lowering the pin
    /// for the old speed as to avoid an extra shift to the "off" state
    /// between speed changes.
    pub async fn on(&mut self, speed: PoolPumpSpeed) -> Result<(), Error> {
        if speed == self.current {
            return Ok(());
        }

        let mut low = self.low.clone();
        let mut med = self.med.clone();
        let mut high = self.high.clone();
        let mut max = self.max.clone();

        let old_speed = self.current;
        self.current = speed;

        let new_speed_result = match speed {
            PoolPumpSpeed::Off => {
                let low_result = low.off().await;
                if let Err(e) = low_result {
                    error!("Error removing pool pump low setting: {}", e);
                };
                let med_result = med.off().await;
                if let Err(e) = med_result {
                    error!("Error removing pool pump med setting: {}", e);
                };
                let high_result = high.off().await;
                if let Err(e) = high_result {
                    error!("Error removing pool pump high setting: {}", e);
                };
                let max_result = max.off().await;
                if let Err(e) = max_result {
                    error!("Error removing pool pump max setting: {}", e);
                };

                Result::Ok(())
            }
            PoolPumpSpeed::Low => low.on().await,
            PoolPumpSpeed::Med => med.on().await,
            PoolPumpSpeed::High => high.on().await,
            PoolPumpSpeed::Max => max.on().await,
        };

        if let Err(e) = new_speed_result {
            error!("Error setting new pool pump speed: {}", e);
        }

        let old_speed_result = match old_speed {
            PoolPumpSpeed::Off => Result::Ok(()),
            PoolPumpSpeed::Low => low.off().await,
            PoolPumpSpeed::Med => med.off().await,
            PoolPumpSpeed::High => high.off().await,
            PoolPumpSpeed::Max => max.off().await,
        };

        if let Err(e) = old_speed_result {
            error!("Error removing old pool pump speed: {}", e);
        }

        Ok(())
    }
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
        pool_pump.on(PoolPumpSpeed::Max).await.unwrap();
        assert_eq!(pool_pump.current, PoolPumpSpeed::Max);
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
