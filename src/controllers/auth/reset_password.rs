use actix_web::{post, web, web::Data, HttpResponse, Responder, Result};
use diesel::RunQueryDsl;

use crate::database::{first, DbPool};
use crate::email;
use crate::models::user::User;
use crate::Settings;

#[post("/reset_password")]
async fn reset_password(
    //identity: Identity,
    email: web::Json<String>,
    db: Data<DbPool>,
    settings: Data<Settings>,
) -> Result<impl Responder> {
    let db_clone = db.clone();

    let user: User = match first!(User::by_email(email.to_string()), User, db.clone()) {
        Ok(user) => user,
        Err(_) => return Ok(HttpResponse::Ok().finish()),
    };

    email::send_password_reset(user, db_clone, settings.mailer_auth_token.clone())
        .await
        // TODO: log error
        .expect("Could not send password reset email");

    Ok(HttpResponse::Ok().finish())
}
