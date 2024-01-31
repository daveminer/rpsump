use std::process::Command;

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
        #[allow(unused)] name: String,
        #[allow(unused)] trigger: Trigger,
        #[allow(unused)] handle: Handle,
        #[allow(unused)] tx: &Sender<Command>,
        #[allow(unused)] delay: u64,
    ) -> Result<(), Error> {
        Ok(())
    }
}
