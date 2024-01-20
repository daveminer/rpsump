use std::{
    process::Command,
    sync::{Arc, Mutex},
};

use crate::hydro::gpio::{InputPin, Level, OutputPin, Pin, Trigger};
use actix_web::rt::Runtime;
use anyhow::Error;
use tokio::sync::mpsc::Sender;

use super::{input_pin::InputPinStub, output_pin::OutputPinStub};

pub struct PinStub {
    pub level: Level,
    pub index: u8,
}

impl Pin for PinStub {
    fn into_input_pullup(self: Box<Self>) -> Box<dyn InputPin> {
        Box::new(InputPinStub {
            level: self.level,
            index: self.index,
        })
    }
    fn into_output_low(self: Box<Self>) -> Box<dyn OutputPin> {
        Box::new(OutputPinStub {
            index: self.index,
            level: Level::Low,
        })
    }
}

impl InputPin for PinStub {
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
        rt: Arc<Mutex<Runtime>>,
        tx: &Sender<Command>,
        delay: u64,
    ) -> Result<(), Error> {
        Ok(())
    }
}

impl OutputPin for PinStub {
    fn set_high(&mut self) {
        self.level = Level::High;
    }

    fn set_low(&mut self) {
        self.level = Level::Low;
    }
}
