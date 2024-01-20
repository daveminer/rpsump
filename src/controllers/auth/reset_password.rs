use actix_web::{post, web, web::Data, HttpRequest, HttpResponse, Responder, Result};
use chrono::Utc;
use diesel::RunQueryDsl;
use serde::Deserialize;
use validator::Validate;

use crate::auth::password::Password;
use crate::config::Settings;
use crate::controllers::auth::{ip_address, validate_password::validate_password};
use crate::database::DbPool;
use crate::models::user::User;
use crate::models::user_event::{EventType, UserEvent};
use crate::util::{spawn_blocking_with_tracing, ApiResponse};

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

/// Request a password reset by sending an email with a reset link to the
/// provided email address.
#[post("/request_password_reset")]
#[tracing::instrument(skip(req, params, db, settings))]
async fn request_password_reset(
    req: HttpRequest,
    params: web::Json<RequestPasswordResetParams>,
    db: Data<dyn DbPool>,
    settings: Data<Settings>,
) -> Result<HttpResponse> {
    let db_clone = db.clone();

    let thread_result = spawn_blocking_with_tracing(move || {
        let mut conn = db_clone.get_conn().expect("Could not get a db connection.");
        User::by_email(params.email.clone()).first::<User>(&mut conn)
    })
    .await;

    let user_lookup = match thread_result {
        Ok(user) => user,
        Err(e) => {
            tracing::error!(
                target = module_path!(),
                error = e.to_string(),
                "User lookup thread failed"
            );
            return Ok(ApiResponse::internal_server_error());
        }
    };

    let user: User = match user_lookup {
        Ok(user) => user,
        Err(e) => {
            tracing::error!(
                target = module_path!(),
                error = e.to_string(),
                "User lookup thread"
            );
            return Ok(ApiResponse::internal_server_error());
        }
    };

    let user_clone = user.clone();
    let ip_addr: String = match ip_address(&req) {
        Ok(ip) => ip,
        Err(e) => {
            tracing::error!(
                target = module_path!(),
                error = e.to_string(),
                "Getting ip address from request failed"
            );
            return Ok(ApiResponse::internal_server_error());
        }
    };

    match UserEvent::create(user_clone, ip_addr, EventType::PasswordReset, db.clone()).await {
        Ok(_) => (),
        Err(e) => {
            tracing::error!(
                target = module_path!(),
                error = e.to_string(),
                "Creating user event for password reset failed"
            );
            return Ok(ApiResponse::internal_server_error());
        }
    }

    match user
        .send_password_reset(
            db.clone(),
            &settings.mailer.server_url,
            req.connection_info().host(),
            &settings.mailer.auth_token.clone(),
        )
        .await
    {
        Ok(_) => (),
        Err(e) => {
            tracing::error!(
                target = module_path!(),
                error = e.to_string(),
                "Sending password reset email failed"
            );
            return Ok(ApiResponse::internal_server_error());
        }
    }

    Ok(ApiResponse::ok(
        "A password reset email will be sent if the email address is valid.".to_string(),
    ))
}

/// Use a token provided in an email to reset a user's password.
#[post("/reset_password")]
#[tracing::instrument(skip(params, db))]
async fn reset_password(
    params: web::Json<ResetPasswordParams>,
    db: Data<dyn DbPool>,
) -> Result<HttpResponse> {
    let token_clone = params.token.clone();

    if let Err(e) = validate_password(&params.new_password) {
        return Ok(ApiResponse::bad_request(e.to_string()));
    }

    let db_clone = db.clone();
    let thread_request = spawn_blocking_with_tracing(move || {
        let mut conn = db_clone.get_conn().expect("Could not get a db connection.");
        User::by_password_reset_token(token_clone.clone()).first::<User>(&mut conn)
    })
    .await;

    let user_lookup = match thread_request {
        Ok(user) => user,
        Err(e) => {
            tracing::error!(
                target = module_path!(),
                error = e.to_string(),
                "Password reset thread failed"
            );
            return Ok(ApiResponse::internal_server_error());
        }
    };

    let user: User = match user_lookup {
        Ok(user) => user,
        Err(_) => return Ok(invalid_token_response()),
    };
    let user_id = user.id;

    if user.password_reset_token_expires_at > Some(Utc::now().naive_utc()) {
        match user.set_password(&params.new_password, db.clone()).await {
            Ok(_) => {
                tracing::info!(target = module_path!(), user_id, "Password reset for user");
                return Ok(ApiResponse::ok("Password reset successfully.".to_string()));
            }
            Err(e) => {
                tracing::error!(
                    target = module_path!(),
                    error = e.to_string(),
                    "Password reset failed"
                );
                return Ok(ApiResponse::internal_server_error());
            }
        }
    }

    Ok(invalid_token_response())
}

fn invalid_token_response() -> HttpResponse {
    ApiResponse::bad_request("Invalid token.".to_string())
}
