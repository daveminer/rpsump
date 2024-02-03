use anyhow::Error;
use tokio::sync::mpsc::Sender;

use crate::hydro::{
    gpio::{InputPin, Level, Trigger},
    signal::Message,
};

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
        #[allow(unused)] message: Message,
        #[allow(unused)] trigger: Trigger,
        #[allow(unused)] tx: &Sender<Message>,
    ) -> Result<(), Error> {
        Ok(())
    }
}
