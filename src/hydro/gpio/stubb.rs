use anyhow::{anyhow, Error};

use crate::hydro::gpio::GpioInterface;

use super::PinInterface;

pub struct GpioStub {
    pub pins: Vec<PinStub>,
}

impl GpioInterface for GpioStub {
    fn get(&self, pin: u8) -> Result<PinStub, Error> {
        match self.pins.iter().find(|p| p.index == pin) {
            Some(pin) => Ok(*pin),
            None => Err(anyhow!("Pin {} not found", pin)),
        }
    }

    fn new() -> Result<Self, Error> {
        Ok(Self { pins: vec![] })
    }

    fn read_pin(&self, pin: u8) -> LevelStub {
        self.read_pin(pin)
    }

    type Level = LevelStub;

    type Pin = PinStub;
}

pub struct PinStub {
    level: LevelStub,
    index: u8,
}

impl PinInterface for PinStub {
    fn into_output_low() {}

    fn into_output_high() {}

    fn pin(&self) -> u8 {
        self.index
    }

    fn read(&self) -> Self::Level {
        self.level
    }

    type Level = LevelStub;
}

pub enum LevelStub {
    Low = 0,
    High = 1,
}

pub struct InputPinStub {}

pub struct OutputPinStub {
    pub level: LevelStub,
}
