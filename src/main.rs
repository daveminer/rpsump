use actix_web::{get, post, App, HttpResponse, HttpServer, Responder};
use rppal::gpio::Level;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio_tungstenite::tungstenite::protocol::Message;

mod board;
mod message;
mod sump;

const CHANNEL_BUFFER_SIZE: usize = 32;

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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let (tx, rx): (Sender<Message>, Receiver<Message>) = channel(CHANNEL_BUFFER_SIZE);

    let board = board::Board::start(tx);
    let sensor_state_clone = Arc::clone(&board.sump.sensor_state);

    let _message_listener_thread =
        tokio::spawn(async { message::listen(sensor_state_clone, rx).await });

    let _sync_reporter_thread = thread::spawn(move || {
        let mut start_time = Instant::now();
        loop {
            println!("{}", board.report());

            // Wait for 5 seconds
            let elapsed_time = start_time.elapsed();
            if elapsed_time < Duration::from_secs(5) {
                thread::sleep(Duration::from_secs(5) - elapsed_time);
            }
            start_time = Instant::now();
        }
    });

    HttpServer::new(|| App::new().service(on).service(off).service(echo))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
