use anyhow::Error;
use rppal::gpio::{Gpio, InputPin, Level, Trigger};
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::tungstenite::protocol::Message;

// Gpio uses BCM pin numbering. BCM GPIO 23 is tied to physical pin 16.
//const GPIO_LOW_SENSOR: u8 = 99;
const HIGH_SENSOR_PIN: u8 = 14; // GPIO #14 == Pin #8

#[derive(Debug)]
pub struct Sump {
    high_sensor: InputPin,
    //low_sensor: InputPin,
    tx: Sender<Message>,
}

impl Sump {
    pub fn new(tx: Sender<Message>) -> Result<Sump, Error> {
        let gpio = Gpio::new()?;

        let mut high_sensor = gpio.get(HIGH_SENSOR_PIN)?.into_input_pullup();
        //let mut low_sensor = gpio.get(low_pin)?.into_input_pullup();

        Self::listen_on_input_pin(&mut high_sensor, "high sensor".to_string(), tx.clone());
        //Self::listen_on_input_pin(&mut low_sensor, "low sensor".to_string(), tx.clone());

        Ok(Sump {
            high_sensor,
            //low_sensor,
            tx,
        })
    }

    fn sump_signal_received(level: Level, sensor_name: String, tx: Sender<Message>) {
        println!("CALLBACK");

        let msg = Message::Text(format!("{sensor_name} to {level}"));
        match tx.blocking_send(msg) {
            Ok(_) => (),
            Err(e) => println!("Error on message tx: {:?}", e),
        };
    }

    fn listen_on_input_pin(pin: &mut InputPin, sensor_name: String, tx: Sender<Message>) {
        pin.set_async_interrupt(Trigger::Both, move |level| {
            println!("INTERRUPT");
            Self::sump_signal_received(level, sensor_name.clone(), tx.clone());
        })
        .expect("Could not not listen on sump pin");
    }
}
