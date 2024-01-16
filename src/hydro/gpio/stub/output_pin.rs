use crate::hydro::gpio::{Level, OutputPin};

pub struct OutputPinStub {
    pub index: u8,
    pub level: Level,
}

impl OutputPin for OutputPinStub {
    fn set_high(&mut self) {
        println!("Set OutputPinStub {} high", self.index);
        self.level = Level::High;
    }

    fn set_low(&mut self) {
        println!("Set OutputPinStub {} low", self.index);
        self.level = Level::Low;
    }
}
