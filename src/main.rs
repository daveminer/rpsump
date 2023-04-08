use actix_web::{get, post, App, HttpResponse, HttpServer, Responder};
use rppal::gpio::Level;
use std::thread;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio_tungstenite::tungstenite::protocol::Message;

mod board;
mod message;
mod sump;

const CHANNEL_BUFFER_SIZE: usize = 32;

pub struct RpSump {
    sump_high_sensor: Level,
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let (tx, rx): (Sender<Message>, Receiver<Message>) = channel(CHANNEL_BUFFER_SIZE);

    let _message_listener_thread = tokio::spawn(async { message::listen(rx).await });

    //let controls = Arc::new(Mutex::new(None));

    let board = board::Board::start(tx);

    //let sump = sump::Sump::new(tx).expect("Could not create sump object");

    //println!("{}", sump.high_sensor.read());

    // let rpsump = Arc::clone(&controls);
    // let mut rpsump_state = rpsump.lock().unwrap();
    // *rpsump_state = Some(RpSump {
    //     sump_high_sensor: sump.high_sensor.read(),
    // });

    let _sync_reporter_thread = thread::spawn(move || {
        let mut start_time = Instant::now();
        loop {
            board.report();

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
