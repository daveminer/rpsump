use crate::sump::Sump;
use actix_web::{get, web::Data, HttpResponse, Responder};

#[get("/info")]
async fn info(_req_body: String, sump: Data<Sump>) -> impl Responder {
    let body = serde_json::to_string(&sump.sensors()).expect("Could not serialize the pin state");

    HttpResponse::Ok().body(body)
}
