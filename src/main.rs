use actix_web::{get, web::Data, App, HttpResponse, HttpServer, Responder};
use std::thread;
use std::time::{Duration, Instant};

use crate::sump::Sump;

mod sump;

struct AppState {
    sump: Sump,
}

#[get("/info")]
async fn info(_req_body: String, data: Data<AppState>) -> impl Responder {
    let pin_state = &data.sump.sensors();

    let body = serde_json::to_string(&pin_state).expect("Could not serialize the pin state");

    HttpResponse::Ok().body(body)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let app_state = Data::new(AppState {
        sump: Sump::new().expect("Could not create sump object"),
    });
    let app_state_clone = app_state.clone();

    let _sync_reporter_thread = thread::spawn(move || {
        let mut start_time = Instant::now();

        loop {
            println!("{:?}", app_state_clone.sump.sensors());

            // Wait for 5 seconds
            let elapsed_time = start_time.elapsed();
            if elapsed_time < Duration::from_secs(5) {
                thread::sleep(Duration::from_secs(5) - elapsed_time);
            }
            start_time = Instant::now();
        }
    });

    HttpServer::new(move || App::new().app_data(app_state.clone()).service(info))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
