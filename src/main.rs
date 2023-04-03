use actix_web::{get, post, App, HttpResponse, HttpServer, Responder};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio_tungstenite::tungstenite::protocol::Message;

mod message;
mod sump;

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
    let (tx, rx): (Sender<Message>, Receiver<Message>) = channel(32);

    let _message_listener_thread = tokio::spawn(async { message::listen(rx) });

    let _sump_thread = tokio::spawn(async {
        sump::Sump::new(23, 99, tx).expect("Could not create sump object");
    });

    HttpServer::new(|| App::new().service(on).service(off).service(echo))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
