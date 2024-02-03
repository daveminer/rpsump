use crate::hydro::gpio::{Level, OutputPin};

pub struct OutputPinStub {
    pub index: u8,
    pub level: Level,
}

impl OutputPin for OutputPinStub {
    fn is_off(&self) -> bool {
        self.level == Level::Low
    }

    fn is_on(&self) -> bool {
        self.level == Level::High
    }

    fn on(&mut self) {
        println!("Set OutputPinStub {} high", self.index);
        self.level = Level::High;
    }

    fn off(&mut self) {
        println!("Set OutputPinStub {} low", self.index);
        self.level = Level::Low;
    }
}
