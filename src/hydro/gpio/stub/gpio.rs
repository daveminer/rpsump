// use anyhow::{anyhow, Error};

// use crate::hydro::gpio::stub::{
//     input_pin::InputPinStub, level::LevelStub, output_pin::OutputPinStub, pin::PinStub,
// };

// pub struct GpioStub {
//     pub pins: Vec<PinStub>,
// }

// impl GpioInterface for GpioStub {
//     type InputPin = InputPinStub;
//     type Level = LevelStub;
//     type OutputPin = OutputPinStub;
//     type Pin = PinStub;

//     fn get(&self, pin: u8) -> Result<PinStub, Error> {
//         match self.pins.iter().find(|p| p.index == pin) {
//             Some(pin) => Ok(*pin),
//             None => Err(anyhow!("Pin {} not found", pin)),
//         }
//     }

//     fn new() -> Result<Self, Error> {
//         Ok(Self { pins: vec![] })
//     }

//     fn read_pin(&self, pin: u8) -> LevelStub {
//         self.read_pin(pin)
//     }
// }
