use actix_web::rt;
use anyhow::Error;
use rppal::gpio::{InputPin, Level, Trigger};
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::tungstenite::protocol::Message;

// Gpio uses BCM pin numbering. BCM GPIO 23 is tied to physical pin 16.
//const GPIO_LOW_SENSOR: u8 = 99;
//const GPIO_HIGH_SENSOR: u8 = 23;

#[derive(Debug)]
pub struct Sump {
    //high_sensor: InputPin,
    //low_sensor: InputPin,
    //tx: Sender<Message>,
}

impl Sump {
    // pub fn new(high_sensor: InputPin, tx: Sender<Message>) -> Result<Sump, Error> {
    //     //let gpio = Gpio::new()?;

    //     //let mut high_sensor = gpio.get(high_pin)?.into_input();
    //     //let mut low_sensor = gpio.get(low_pin)?.into_input();

    //     Self::listen_on_input_pin(high_sensor, "high sensor".to_string(), tx.clone());
    //     //Self::listen_on_input_pin(&mut low_sensor, "low sensor".to_string(), tx.clone());

    //     Ok(Sump {
    //         high_sensor,
    //         //low_sensor,
    //         tx,
    //     })
    // }

    // fn sump_signal_received(level: Level, sensor_name: String, tx: Sender<Message>) {
    //     println!("CALLBACK");

    //     let msg = Message::Text(format!("{sensor_name} to {level}"));
    //     rt::spawn(async move { tx.send(msg).await });
    // }

    //fn listen_on_input_pin(pin: InputPin, sensor_name: String, tx: Sender<Message>) {
    //pin.set_async_interrupt(Trigger::Both, move |level| {
    //    Self::sump_signal_received(level, sensor_name.clone(), tx.clone());
    //})
    //.expect("Could not not listen on sump pin");
    //}
}
