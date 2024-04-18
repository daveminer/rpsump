use anyhow::Error;
use mockall::automock;
use std::fmt;
use tokio::sync::mpsc::Sender;

use crate::hydro::signal::Message;

pub mod rppal;

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
}

#[automock]
pub trait Pin: Send + Sync {
    fn into_input_pullup(self: Box<Self>) -> Box<dyn InputPin>;
    fn into_output_low(self: Box<Self>) -> Box<dyn OutputPin>;
}

#[automock]
pub trait InputPin: Send + Sync {
    fn is_high(&self) -> bool;
    fn is_low(&self) -> bool;
    fn read(&self) -> Level;
    fn set_async_interrupt(
        &mut self,
        message: Message,
        trigger: Trigger,
        tx: &Sender<Message>,
    ) -> Result<(), Error>;
}

impl fmt::Debug for dyn InputPin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InputPin")
            .field("is_high", &self.is_high())
            .field("is_low", &self.is_low())
            .field("read", &self.read())
            .finish()
    }
}

#[automock]
pub trait OutputPin: Send + Sync {
    fn is_on(&self) -> bool;
    fn is_off(&self) -> bool;

    fn on(&mut self);
    fn off(&mut self);
}
