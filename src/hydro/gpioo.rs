use anyhow::{anyhow, Error};
use rppal::gpio::{Gpio as RppalGpio, Level, Pin, Trigger};

pub trait Gpio {
    fn get(&self, pin: u8) -> Result<Pin, Error>;
    fn new() -> Result<Self, Error>
    where
        Self: std::marker::Sized;
    fn read_pin(&self, pin: u8) -> Level;
}

impl GpioInterface for RppalGpio {
    fn get(&self, pin: u8) -> Result<Pin, Error> {
        self.get(pin).map_err(|e| anyhow!(e.to_string()))
    }

    fn new() -> Result<Self, Error> {
        RppalGpio::new().map_err(|e| anyhow!(e.to_string()))
    }

    fn read_pin(&self, pin: u8) -> Level {
        self.read_pin(pin)
    }
}

pub struct GpioStub {
    pub pins: Vec<PinStub>,
}

pub struct PinStub {
    level: Level,
    index: u8,
}

impl Gpio for GpioStub {
    fn get(&self, pin: u8) -> Result<Pin, Error> {
        match self.pins.get(pin) {
            Some(pin) => Ok(pin.clone()),
            None => Err(anyhow!("Pin {} not found", pin)),
        }
        self.pins.push(PinStub::new(pin)?);
        Ok(PinStub::new(pin))
    }

    fn new() -> Result<Self, Error> {
        Ok(Self { pins: vec![] })
    }

    fn read_pin(&self, pin: u8) -> Level {
        self.read_pin(pin)
    }
}
