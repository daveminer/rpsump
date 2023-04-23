use actix_web::Result;
use actix_web::{get, HttpResponse, Responder};

#[get("/info")]
//async fn info(_req_body: String, sump: Data<Sump>) -> Result<impl Responder> {
async fn info(_req_body: String) -> Result<impl Responder> {
    //let body = serde_json::to_string(&sump.sensors()).expect("Could not serialize the pin state");

    //Ok(HttpResponse::Ok().body(body))
    Ok(HttpResponse::Ok().body("Sump disabled."))
}
