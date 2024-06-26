use anyhow::Error;
use std::{sync::Arc, time::Duration};
use tokio::{
    runtime::Handle,
    sync::{mpsc::Sender, Mutex},
};

use crate::hydro::{
    debounce::Debouncer,
    gpio::{Gpio, InputPin, OutputPin, Pin, Trigger},
    signal::{Message, Signal},
};

impl Gpio for rppal::gpio::Gpio {
    fn get(&self, pin: u8) -> Result<Box<dyn Pin>, Error> {
        let pin = self.get(pin).map(|p| Box::new(p) as Box<dyn Pin>)?;

        Ok(pin)
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
        tx: &Sender<Signal>,
        delay: Duration,
        handle: Handle,
    ) -> Result<(), Error> {
        let message = message.clone();
        let tx = tx.clone();
        //let debouncer: Arc<Mutex<Debouncer>> = Arc::new(Mutex::new(None));
        let debouncer = Arc::from(Mutex::new(Debouncer::new(
            delay,
            message.clone(),
            tx.clone(),
        )));

        let callback = move |level: rppal::gpio::Level| {
            handle.block_on(handle_interrupt(Arc::clone(&debouncer), level));
        };

        Ok(self.set_async_interrupt(trigger.into(), callback)?)
    }
}

async fn handle_interrupt(debouncer: Arc<Mutex<Debouncer>>, level: rppal::gpio::Level) {
    let mut debouncer = debouncer.lock().await;
    let running = debouncer.running.lock().await;
    if *running {
        drop(running);
        debouncer.reset_deadline(level.into()).await;
    } else {
        drop(running);
        debouncer.start().await;
    }
}

impl From<Trigger> for rppal::gpio::Trigger {
    fn from(val: Trigger) -> Self {
        match val {
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
