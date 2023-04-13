use actix_web::{get, post, App, HttpResponse, HttpServer, Responder};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio_tungstenite::tungstenite::protocol::Message;

mod board;
mod message;
mod sump;

const CHANNEL_BUFFER_SIZE: usize = 32;

#[post("/")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Aggregates messages from GPIO events
    let (tx, rx): (Sender<Message>, Receiver<Message>) = channel(CHANNEL_BUFFER_SIZE);

    // Singleton reference to the GPIO interface; this is expected to live
    // for the lifetime of the process.
    let board = board::Board::start(tx);

    //
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

    HttpServer::new(|| App::new().service(echo))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
