use anyhow::Error;

use crate::hydro::gpio::{InputPin, InputPinCallback, Level, Trigger};

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
        _trigger: Trigger,
        _callback: InputPinCallback,
    ) -> Result<(), Error> {
        Ok(())
    }
}
