use crate::hydro::gpio::OutputPin;

pub struct OutputPinStub {
    pub index: u8,
}

impl OutputPin for OutputPinStub {
    fn set_high(&mut self) {
        println!("Set pin {} high", self.index);
    }

    fn set_low(&mut self) {
        println!("Set pin {} low", self.index);
    }
}
