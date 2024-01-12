use crate::hydro::gpio::{InputPin, InputPinCallback, Level, OutputPin, Pin, Trigger};
use anyhow::Error;

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
        Box::new(OutputPinStub { index: self.index })
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
        trigger: Trigger,
        callback: InputPinCallback,
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
