use anyhow::{anyhow, Error};

use crate::hydro::{
    self,
    gpio::{Gpio, InputPin, OutputPin, Pin},
    Level,
};

// impl GpioInterface for Gpio {
//     type InputPin = InputPin;
//     type Level = Level;
//     type OutputPin = OutputPin;
//     type Pin = Pin;
//     fn get(&self, pin: u8) -> Result<Pin, Error> {
//         self.get(pin).map_err(|e| anyhow!(e.to_string()))
//     }

//     fn new() -> Result<Self, Error> {
//         Gpio::new().map_err(|e| anyhow!(e.to_string()))
//     }

//     fn read_pin(&self, pin: u8) -> Level {
//         self.read_pin(pin)
//     }
// }

impl Gpio for rppal::gpio::Gpio {
    fn get(&self, pin: u8) -> Result<Box<dyn Pin>, Error> {
        let pin = self.get(pin).map(|p| Box::new(p) as Box<dyn Pin>)?;

        Ok(pin)
    }

    fn create() -> Result<Self, Error> {
        rppal::gpio::Gpio::new().map_err(|e| anyhow!(e.to_string()))
    }
}

// impl PinInterface for Pin {
//     type InputPin = InputPin;
//     type Level = Level;
//     type OutputPin = OutputPin;

//     fn into_input(self) -> InputPin {
//         self.into_input()
//     }

//     fn into_input_pullup(self) -> Self::InputPin {
//         self.into_input_pullup()
//     }

//     fn into_output_low(self) -> OutputPin {
//         self.into_output_low()
//     }

//     fn into_output_high(self) -> OutputPin {
//         self.into_output_high()
//     }

//     fn pin(&self) -> u8 {
//         self.pin()
//     }

//     fn read(&self) -> Level {
//         self.read()
//     }
// }

impl Pin for rppal::gpio::Pin {
    fn into_input_pullup(self: Box<Self>) -> Box<dyn InputPin> {
        self.into_input_pullup()
    }
    fn into_output_low(self: Box<Self>) -> Box<dyn OutputPin> {
        self.into_output_low()
    }
}

impl Into<Level> for rppal::gpio::Level {
    fn into(self) -> Level {
        match self {
            rppal::gpio::Level::Low => Level::Low,
            rppal::gpio::Level::High => Level::High,
        }
    }
}

impl InputPin for rppal::gpio::Pin {
    fn is_high(&self) -> bool {
        self.is_high()
    }

    fn is_low(&self) -> bool {
        self.is_low()
    }

    fn read(&self) -> Level {
        self.read().into()
    }

    fn set_async_interrupt(
        &mut self,
        trigger: hydro::gpio::Trigger,
        callback: super::InputPinCallback,
    ) -> Result<(), Error> {
        self.set_async_interrupt(trigger, callback)
            .map_err(|e| anyhow!(e.to_string()))
    }
}

impl OutputPin for rppal::gpio::Pin {
    fn set_high(&mut self) {
        self.set_high()
    }

    fn set_low(&mut self) {
        self.set_low()
    }
}

// impl InputPinInterface for InputPin {
//     fn is_high(&self) -> bool {
//         self.is_high()
//     }

//     fn is_low(&self) -> bool {
//         self.is_low()
//     }
// }

// impl OutputPinInterface for OutputPin {
//     fn set_high(&mut self) {
//         self.set_high()
//     }

//     fn set_low(&mut self) {
//         self.set_low()
//     }
// }

// impl LevelInterface for Level {
//     fn is_high(&self) -> bool {
//         self.is_high()
//     }

//     fn is_low(&self) -> bool {
//         self.is_low()
//     }
// }
