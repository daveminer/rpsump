use actix_web::{get, post, App, HttpResponse, HttpServer, Responder};
use rppal::gpio::{Gpio, InputPin, Level, Trigger};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio_tungstenite::tungstenite::protocol::Message;

mod message;
mod sump;

const CHANNEL_BUFFER_SIZE: usize = 32;

// Gpio uses BCM pin numbering. BCM GPIO 23 is tied to physical pin 16.
const GPIO_LED: u8 = 23;

// #[get("/")]
// async fn index() -> impl Responder {
//     //let gpio = Gpio::new()?;
// }

#[get("/on")]
async fn on() -> impl Responder {
    //let gpio = Gpio::new()?;

    //let mut pin = Gpio::new()?.get(GPIO_LED)?.into_output();

    //pin.set_high();

    HttpResponse::Ok().body("On!")
}

#[get("/off")]
async fn off() -> impl Responder {
    HttpResponse::Ok().body("Off!")
}

#[post("/")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

// Gpio uses BCM pin numbering. BCM GPIO 23 is tied to physical pin 16.
//const GPIO_LED: u8 = 23;

// fn main() {
//     println!("Hello, world!");

//     let gpio = Gpio::new()?;

//     let mut pin = Gpio::new()?.get(GPIO_LED)?.into_output();

//     pin.set_high();
// }

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let (tx, rx): (Sender<Message>, Receiver<Message>) = channel(CHANNEL_BUFFER_SIZE);
    let high_pin = 14; // GPIO #14 == Pin #8
                       //let low_pin = 5;

    let _message_listener_thread = tokio::spawn(async { message::listen(rx) });

    let gpio = Gpio::new().expect("Could not create gpio device");
    let mut high_sensor = gpio
        .get(high_pin)
        .expect("Could not get high pin")
        .into_input_pullup();
    //let mut low_sensor = gpio.get(low_pin)?.into_input();

    high_sensor
        .set_async_interrupt(Trigger::Both, move |level| {
            println!("LEVEL: {}", level);
            //Self::sump_signal_received(level, sensor_name.clone(), tx.clone());
        })
        .expect("Could not not listen on sump pin");

    //let _sump = sump::Sump::new(14, tx).expect("Could not create sump object");

    HttpServer::new(|| App::new().service(on).service(off).service(echo))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
