use actix_web::{post, web, web::Data, HttpResponse, Responder, Result};
use diesel::RunQueryDsl;

use crate::database::{first, DbPool};
use crate::email;
use crate::models::user::User;
use crate::Settings;

#[post("/verify_email")]
async fn verify_email(
    token: web::Json<String>,
    db: Data<DbPool>,
    settings: Data<Settings>,
) -> Result<impl Responder> {
    match first!(User::verify_email(token), User, db) {
        Ok() => Ok(HttpResponse::Ok().body(json!({ "message": "Email verified." }))),
        Err(_) => Ok(HttpResponse::BadRequest().body(json!({ "message": "Invalid token." }))),
    }
}
