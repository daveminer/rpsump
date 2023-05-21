use actix_identity::Identity;
use actix_web::{post, HttpResponse, Responder};

//#[post("/logout")]
async fn logout(identity: Identity) -> impl Responder {
    Identity::logout(identity);

    HttpResponse::Ok().finish()
}
