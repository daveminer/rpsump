use actix_web::{get, post, web, web::Data, HttpRequest, HttpResponse, Responder, Result};
use diesel::RunQueryDsl;
use serde::Deserialize;

use crate::auth::{password::Password, validate_password};
use crate::config::Settings;
use crate::database::{first, DbPool};
use crate::models::user::User;
use crate::models::user_event::{EventType, UserEvent};

#[derive(Deserialize)]
pub struct ResetPasswordParams {
    pub token: String,
    pub new_password: Password,
    pub new_password_confirmation: Password,
}

// Request a password reset by sending an email with a reset link to the
// provided email address.
#[post("/reset_password")]
async fn request_password_reset(
    email: String,
    req: HttpRequest,
    db: Data<DbPool>,
    settings: Data<Settings>,
) -> Result<impl Responder> {
    let db_clone = db.clone();
    let db_clone_two = db.clone();
    let user: User = match first!(User::by_email(email.clone()), User, db_clone) {
        Ok(user) => user,
        Err(_) => return Ok(password_reset_response()),
    };

    let user_clone = user.clone();

    let conn_info = req.peer_addr().expect("Could not get IP address.");
    let ip_addr = conn_info.ip().to_string();

    UserEvent::create(user_clone, ip_addr, EventType::PasswordReset, db_clone_two)
        .await
        .expect("Could not create user event.");

    user.send_password_reset(
        db.clone(),
        req.connection_info().host().to_string(),
        settings.mailer_auth_token.clone(),
    )
    .await
    .expect("Could not send password reset email");

    Ok(password_reset_response())
}

fn password_reset_response() -> HttpResponse {
    HttpResponse::Ok().body("A password reset email will be sent if the email address is valid.")
}

// Use a token provided in an email to reset a user's password.
#[get("/new_password")]
async fn reset_password(
    params: web::Query<ResetPasswordParams>,
    db: Data<DbPool>,
) -> Result<impl Responder> {
    //let password_confirmation = params.new_password_confirmation.clone();
    let token_clone = params.token.clone();

    if let Err(e) = validate_password(&params.new_password) {
        return Ok(HttpResponse::BadRequest().body(e.to_string()));
    }

    let db_clone = db.clone();

    let user: User = match first!(User::by_password_reset_token(token_clone), User, db.clone()) {
        Ok(user) => user,
        Err(_) => return Ok(HttpResponse::BadRequest().body("Invalid token.")),
    };

    user.set_password(&params.new_password, db_clone)
        .await
        .expect("Could not set password.");

    Ok(HttpResponse::Ok().finish())
}
