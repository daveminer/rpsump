use actix_web::{get, post, App, HttpResponse, HttpServer, Responder};
use std::sync::Mutex;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio_tungstenite::tungstenite::protocol::Message;

mod message;
mod sump;

const CHANNEL_BUFFER_SIZE: usize = 32;

static RPSUMP: Mutex<Option<sump::Sump>> = Mutex::new(None);

pub struct RpSump {
    sump_high_sensor: bool,
}

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

    let _message_listener_thread = tokio::spawn(async { message::listen(rx).await });

    let sump = sump::Sump::new(tx).expect("Could not create sump object");

    println!("{}", sump.high_sensor.read());

    //let high_sensor_on = if sump.high_sensor.read() == Level::HIGH { true } else {false};

    //let mut rpsump = RPSUMP.lock().unwrap();
    //*rpsump = RpSump{ sump_high_sensor: sump.high_}

    HttpServer::new(|| App::new().service(on).service(off).service(echo))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
