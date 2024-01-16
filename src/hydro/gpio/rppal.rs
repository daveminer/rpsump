use anyhow::{anyhow, Error};

use crate::hydro::{
    gpio::{Gpio, InputPin, OutputPin, Pin, Trigger},
    Level,
};

impl Gpio for rppal::gpio::Gpio {
    fn get(&self, pin: u8) -> Result<Box<dyn Pin>, Error> {
        let pin = self.get(pin).map(|p| Box::new(p) as Box<dyn Pin>)?;

        Ok(pin)
    }

    fn create() -> Result<Self, Error> {
        rppal::gpio::Gpio::new().map_err(|e| anyhow!(e.to_string()))
    }
}

impl Pin for rppal::gpio::Pin {
    fn into_input_pullup(self: Box<Self>) -> Box<dyn InputPin> {
        Box::new(rppal::gpio::Pin::into_input_pullup(*self))
    }
    fn into_output_low(self: Box<Self>) -> Box<dyn OutputPin> {
        Box::new(rppal::gpio::Pin::into_output_low(*self))
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

impl InputPin for rppal::gpio::InputPin {
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
        trigger: Trigger,
        mut callback: super::InputPinCallback,
    ) -> Result<(), Error> {
        let converted_callback = Box::new(move |level: rppal::gpio::Level| callback(level.into()));

        self.set_async_interrupt(trigger.into(), converted_callback)
            .map_err(|e| anyhow!(e.to_string()))
    }
}

impl Into<rppal::gpio::Trigger> for Trigger {
    fn into(self) -> rppal::gpio::Trigger {
        match self {
            Trigger::Disabled => rppal::gpio::Trigger::Disabled,
            Trigger::RisingEdge => rppal::gpio::Trigger::RisingEdge,
            Trigger::FallingEdge => rppal::gpio::Trigger::FallingEdge,
            Trigger::Both => rppal::gpio::Trigger::Both,
        }
    }
}

impl OutputPin for rppal::gpio::OutputPin {
    fn set_high(&mut self) {
        self.set_high()
    }

    fn set_low(&mut self) {
        self.set_low()
    }
}
