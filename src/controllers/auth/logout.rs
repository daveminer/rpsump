use actix_identity::Identity;
use actix_web::{post, HttpResponse, Responder};

#[post("/logout")]
async fn logout(identity: Identity) -> impl Responder {
    // TODO: store token in cache or database

    // TODO: send email to user with the link to the reset password page including the token

    //identity.forget();
    Identity::logout(identity);

    HttpResponse::Ok().finish()
}
