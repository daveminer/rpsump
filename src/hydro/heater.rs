use crate::hydro::gpio::{Gpio, Level};
use crate::{config::HeaterConfig, hydro::control::Control};
use anyhow::Error;

#[derive(Clone)]
pub struct Heater {
    pub control: Control,
}

impl Heater {
    pub fn new<G>(config: &HeaterConfig, gpio: &G) -> Result<Self, Error>
    where
        G: Gpio,
    {
        let control = Control::new("pool pump".into(), config.control_pin, gpio)?;
        Ok(Self { control })
    }

    pub async fn on(&mut self) -> Result<(), Error> {
        self.control.level = Level::High;
        let mut lock = self.control.lock().await;
        lock.set_high();

        Ok(())
    }

    pub async fn off(&mut self) -> Result<(), Error> {
        self.control.level = Level::Low;
        let mut lock = self.control.lock().await;
        lock.set_low();

        Ok(())
    }

    pub fn is_on(&self) -> bool {
        self.control.level == Level::High
    }

    pub fn is_off(&self) -> bool {
        self.control.level == Level::Low
    }
}

#[cfg(test)]
mod tests {
    use crate::{config::HeaterConfig, hydro::heater::Heater, test_fixtures::gpio::mock_gpio_get};

    #[tokio::test]
    async fn test_heater_on() {
        let config = HeaterConfig { control_pin: 1 };
        let mock_gpio = mock_gpio_get(vec![1]);

        let mut heater = Heater::new(&config, &mock_gpio).unwrap();
        let _ = heater.on().await.unwrap();

        assert_eq!(heater.is_on(), true);
    }

    #[tokio::test]
    async fn test_heater_off() {
        let config = HeaterConfig { control_pin: 1 };
        let mock_gpio = mock_gpio_get(vec![1]);

        let heater = Heater::new(&config, &mock_gpio).unwrap();

        assert_eq!(heater.is_off(), true);
    }
}
