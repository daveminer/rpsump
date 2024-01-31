use anyhow::{anyhow, Error};
use std::{
    process::Command,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{runtime::Handle, sync::mpsc::Sender};

use crate::hydro::{
    debounce::Debouncer,
    gpio::{Gpio, InputPin, OutputPin, Pin, Trigger},
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

    fn set_async_interrupt(
        &mut self,
        name: String,
        trigger: Trigger,
        handle: Handle,
        tx: &Sender<Command>,
        delay: u64,
    ) -> Result<(), Error> {
        //let rt = rt.borrow_mut();
        // let debouncer = Arc::clone(&debouncer);
        // let rt = rt.clone();
        // let tx = tx.clone();
        let debouncer: Arc<Mutex<Option<Debouncer>>> = Arc::from(Mutex::new(None));
        let tx = tx.clone();
        let callback = move |level: rppal::gpio::Level| {
            let debouncer = Arc::clone(&debouncer);
            //let handle = rt.handle().clone();
            callback(level, &name, debouncer, delay, handle.clone(), &tx);
        };
        Ok(self.set_async_interrupt(trigger.into(), callback)?)
    }
}

fn callback(
    level: rppal::gpio::Level,
    _name: &str,
    debouncer: Arc<Mutex<Option<Debouncer>>>,
    delay: u64,
    rt: Handle,
    _tx: &Sender<Command>,
) {
    // let level = level.into();
    // let name = name.to_string();
    // let tx = tx.clone();
    let shared_deb = Arc::clone(&debouncer);
    let mut deb = shared_deb.lock().unwrap();

    if deb.is_some() {
        deb.as_mut().unwrap().reset_deadline(level.into());
        return;
    }

    let debouncer = Debouncer::new(level.into(), Duration::new(2, 0));
    *deb = Some(debouncer);

    let sleep = deb.as_ref().unwrap().sleep();
    //let rt = Runtime::new().unwrap();
    rt.block_on(sleep);
    *deb = None;
    drop(deb);

    // TODO: send tx message
    // rt.block_on(update_irrigation_low_sensor(
    //     level,
    //     Arc::clone(&irrigation_pump_control_pin),
    //     Arc::clone(&sensor_state),
    //     delay,
    // ));
    if delay > 0 {
        rt.block_on(tokio::time::sleep(Duration::from_secs(delay as u64)));
    }

    // tokio::spawn(async move {
    //     let cmd = Command::new("sleep").arg(delay.to_string()).output();
    //     if let Err(e) = cmd {
    //         log::error!("Error running sleep command: {}", e);
    //     }
    //     if let Err(e) = tx.send(Command::new("irrigator").arg(name).arg(level)) {
    //         log::error!("Error sending command: {}", e);
    //     }
    // });
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

impl From<rppal::gpio::Level> for crate::hydro::Level {
    fn from(level: rppal::gpio::Level) -> Self {
        match level {
            rppal::gpio::Level::Low => crate::hydro::Level::Low,
            rppal::gpio::Level::High => crate::hydro::Level::High,
        }
    }
}
