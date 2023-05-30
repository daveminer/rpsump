use actix_web::{post, web, web::Data, HttpRequest, HttpResponse, Responder, Result};
use chrono::Utc;
use diesel::RunQueryDsl;
use serde::Deserialize;
use validator::Validate;

use crate::auth::{password::Password, validate_password};
use crate::config::Settings;
use crate::controllers::ApiResponse;
use crate::database::{first, DbPool};
use crate::models::user::User;
use crate::models::user_event::{EventType, UserEvent};

#[derive(Deserialize, Validate)]
pub struct ResetPasswordParams {
    pub token: String,
    #[validate(custom = "validate_password")]
    pub new_password: Password,
    #[validate(must_match(
        other = "new_password",
        message = "Password and confirm password must match."
    ))]
    pub new_password_confirmation: Password,
}

#[derive(Deserialize)]
pub struct RequestPasswordResetParams {
    pub email: String,
}

// Request a password reset by sending an email with a reset link to the
// provided email address.
#[post("/request_password_reset")]
#[tracing::instrument(skip(req, params, db, settings))]
async fn request_password_reset(
    req: HttpRequest,
    params: web::Json<RequestPasswordResetParams>,
    db: Data<DbPool>,
    settings: Data<Settings>,
) -> Result<impl Responder> {
    let db_clone = db.clone();
    let db_clone_two = db.clone();
    let user: User = match first!(User::by_email(params.email.clone()), User, db_clone) {
        Ok(user) => user,
        Err(_) => return Ok(ApiResponse::internal_server_error()),
    };

    let user_clone = user.clone();
    let conn_info = req.peer_addr().expect("Could not get IP address.");
    let ip_addr = conn_info.ip().to_string();
    UserEvent::create(user_clone, ip_addr, EventType::PasswordReset, db_clone_two)
        .await
        .expect("Could not create user event.");

    user.send_password_reset(
        db.clone(),
        &settings.mailer.server_url,
        req.connection_info().host(),
        &settings.mailer.auth_token.clone(),
    )
    .await
    .expect("Could not send password reset email");

    Ok(ApiResponse::ok(
        "A password reset email will be sent if the email address is valid.".to_string(),
    ))
}

// Use a token provided in an email to reset a user's password.
#[post("/reset_password")]
#[tracing::instrument(skip(params, db))]
async fn reset_password(
    params: web::Json<ResetPasswordParams>,
    db: Data<DbPool>,
) -> Result<impl Responder> {
    //let password_confirmation = params.new_password_confirmation.clone();
    let token_clone = params.token.clone();

    if let Err(e) = validate_password(&params.new_password) {
        return Ok(ApiResponse::bad_request(e.to_string()));
    }

    let db_clone = db.clone();

    let user: User = match first!(User::by_password_reset_token(token_clone), User, db.clone()) {
        Ok(user) => user,
        Err(_) => return Ok(invalid_token_response()),
    };

    if user.password_reset_token_expires_at > Some(Utc::now().naive_utc()) {
        user.set_password(&params.new_password, db_clone)
            .await
            .expect("Could not set password.");

        return Ok(ApiResponse::ok("Password reset successfully.".to_string()));
    }

    Ok(invalid_token_response())
}

fn invalid_token_response() -> HttpResponse {
    ApiResponse::bad_request("Invalid token.".to_string())
}
