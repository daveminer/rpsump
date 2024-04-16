use anyhow::{anyhow, Error};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::mpsc::Sender;

use crate::hydro::{
    debounce::Debouncer,
    gpio::{Gpio, InputPin, OutputPin, Pin, Trigger},
    signal::Message,
};

impl Gpio for rppal::gpio::Gpio {
    fn get(&self, pin: u8) -> Result<Box<dyn Pin>, Error> {
        let pin = self.get(pin).map(|p| Box::new(p) as Box<dyn Pin>)?;

        Ok(pin)
    }

    // TODO: used?
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

impl InputPin for rppal::gpio::InputPin {
    fn is_high(&self) -> bool {
        self.is_high()
    }

    fn is_low(&self) -> bool {
        self.is_low()
    }

    fn read(&self) -> crate::hydro::Level {
        self.read().into()
    }

    /// Wrapper around rppal's set_async_interrupt to allow for use of a shared
    /// debouncer for each interrupt
    ///
    /// # Arguments
    ///
    /// * `name`    - The name of the pin; for logging and reporting
    /// * `trigger` - The level(s) that trigger the interrupt
    /// * `handle`  - The tokio runtime handle to use for async operations
    /// * `tx`      - The channel to report triggers to the main channel
    /// * `delay`   - The delay to use for debouncing the interrupt
    fn set_async_interrupt(
        &mut self,
        message: Message,
        trigger: Trigger,
        tx: &Sender<Message>,
    ) -> Result<(), Error> {
        let message = message.clone();
        let tx = tx.clone();

        let callback = move |level: rppal::gpio::Level| {
            // Create a debouncer instance for this interrupt
            let debouncer: Arc<Mutex<Option<Debouncer>>> = Arc::from(Mutex::new(None));

            callback(level, &message, debouncer, &tx);
        };

        Ok(self.set_async_interrupt(trigger.into(), callback)?)
    }
}

fn callback(
    level: rppal::gpio::Level,
    message: &Message,
    debouncer: Arc<Mutex<Option<Debouncer>>>,
    tx: &Sender<Message>,
) {
    let shared_deb = Arc::clone(&debouncer);
    let mut deb = shared_deb.lock().unwrap();

    // If the debouncer is already present, reset the deadline and return
    if deb.is_some() {
        deb.as_mut().unwrap().reset_deadline(level.into());
        return;
    }

    let debouncer = Debouncer::new(
        level.into(),
        Duration::new(2, 0),
        message.clone(),
        tx.clone(),
    );
    *deb = Some(debouncer);

    *deb = None;
    drop(deb);
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
    fn is_on(&self) -> bool {
        self.is_set_high()
    }

    fn is_off(&self) -> bool {
        self.is_set_low()
    }

    fn on(&mut self) {
        self.set_high()
    }

    fn off(&mut self) {
        self.set_low()
    }
}

impl From<rppal::gpio::Level> for crate::hydro::Level {
    fn from(level: rppal::gpio::Level) -> Self {
        match level {
            rppal::gpio::Level::Low => crate::hydro::Level::Low,
            rppal::gpio::Level::High => crate::hydro::Level::High,
        }
    }
}
