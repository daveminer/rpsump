use anyhow::Error;
use mockall::*;

pub mod rppal;
pub mod stub;

type InputPinCallback = Box<dyn FnMut(Level) + Send>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Level {
    Low = 0,
    High = 1,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Trigger {
    Disabled,
    RisingEdge,
    FallingEdge,
    Both,
}

// pub trait GpioInterface {
//     type Pin: PinInterface;
//     type InputPin: InputPinInterface;
//     type Level;
//     type OutputPin: OutputPinInterface;

//     fn get(&self, pin: u8) -> Result<Self::Pin, Error>;
//     fn new() -> Result<Self, Error>
//     where
//         Self: std::marker::Sized;
//     fn read_pin(&self, pin: u8) -> Self::Level;
// }

#[automock]
pub trait Gpio {
    fn get(&self, pin: u8) -> Result<Box<dyn Pin>, Error>;

    fn create() -> Result<Self, Error>
    where
        Self: std::marker::Sized;
}

// pub trait PinInterface {
//     type InputPin: InputPinInterface;
//     type Level;
//     type OutputPin: OutputPinInterface;

//     fn into_input(self) -> Self::InputPin;
//     fn into_input_pullup(self) -> Self::InputPin;
//     fn into_output_low(self) -> Self::OutputPin;
//     fn into_output_high(self) -> Self::OutputPin;
//     fn pin(&self) -> u8;
//     fn read(&self) -> Self::Level;
// }

#[automock]
pub trait Pin: Send + Sync {
    fn into_input_pullup(self: Box<Self>) -> Box<dyn InputPin>;
    fn into_output_low(self: Box<Self>) -> Box<dyn OutputPin>;
}

// pub trait LevelInterface {}

// TODO: check traits
pub trait InputPin: Send + Sync {
    fn is_high(&self) -> bool;
    fn is_low(&self) -> bool;
    fn read(&self) -> Level;
    fn set_async_interrupt(
        &mut self,
        trigger: Trigger,
        callback: InputPinCallback,
    ) -> Result<(), Error>;
}
// TODO: check traits
pub trait OutputPin: Send + Sync {
    fn set_high(&mut self);
    fn set_low(&mut self);
}
