use anyhow::Error;
use mockall::*;

pub mod rppal;
pub mod stub;

type InputPinCallback = Box<dyn FnMut(Level) + Send>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Level {
    Low = 0,
    High = 1,
    Both = 2,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Trigger {
    Disabled,
    RisingEdge,
    FallingEdge,
    Both,
}

#[automock]
pub trait Gpio {
    fn get(&self, pin: u8) -> Result<Box<dyn Pin>, Error>;

    fn create() -> Result<Self, Error>
    where
        Self: std::marker::Sized;
}

#[automock]
pub trait Pin: Send + Sync {
    fn into_input_pullup(self: Box<Self>) -> Box<dyn InputPin>;
    fn into_output_low(self: Box<Self>) -> Box<dyn OutputPin>;
}

// TODO: check traits
#[automock]
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
#[automock]
pub trait OutputPin: Send + Sync {
    fn set_high(&mut self);
    fn set_low(&mut self);
}
