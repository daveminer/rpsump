use std::{
    process::Command,
    sync::{Arc, Mutex},
};

use actix_web::rt::Runtime;
use anyhow::Error;
use tokio::{runtime::Handle, sync::mpsc::Sender};

use crate::hydro::gpio::{InputPin, Level, Trigger};

pub struct InputPinStub {
    pub level: Level,
    pub index: u8,
}

impl InputPin for InputPinStub {
    fn is_high(&self) -> bool {
        self.level == Level::High
    }

    fn is_low(&self) -> bool {
        self.level == Level::Low
    }

    fn read(&self) -> Level {
        self.level
    }

    fn set_async_interrupt(
        &mut self,
        name: String,
        trigger: Trigger,
        handle: Handle,
        tx: &Sender<Command>,
        delay: u64,
    ) -> Result<(), Error> {
        Ok(())
    }
}
