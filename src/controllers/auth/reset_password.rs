use actix_web::{post, web, web::Data, HttpRequest, HttpResponse, Result};
use chrono::Utc;
use diesel::RunQueryDsl;
use serde::Deserialize;
use validator::Validate;

use crate::auth::password::Password;
use crate::auth::token::Token;
use crate::config::Settings;
use crate::controllers::auth::helpers::{
    error_response, invalid_token_response, ip_address, validate_password_strength,
};
use crate::models::user::User;
use crate::repository::Repo;
use crate::util::{spawn_blocking_with_tracing, ApiResponse};

#[derive(Deserialize, Validate)]
pub struct ResetPasswordParams {
    pub token: String,
    #[validate(custom = "validate_password_strength")]
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
#[tracing::instrument(skip(req, params, repo, settings))]
async fn request_password_reset(
    req: HttpRequest,
    params: web::Json<RequestPasswordResetParams>,
    repo: Data<Repo>,
    settings: Data<Settings>,
) -> Result<HttpResponse> {
    let maybe_user = match repo.user_by_email(params.email.clone()).await {
        Ok(maybe_user) => maybe_user,
        Err(e) => error_response(e, "User lookup failed"),
    };

    let ip_addr: String = match ip_address(&req) {
        Ok(ip) => ip,
        Err(e) => error_response(e, "Getting ip address from request failed"),
    };

    if let Some(user) = maybe_user {
        let token = Token::new_password_reset(user.id);

        let user = repo.password_reset(user, token).await;

        user.send_password_reset(
            &settings.mailer.server_url,
            req.connection_info().host(),
            &settings.mailer.auth_token.clone(),
        )
    };

    Ok(ApiResponse::ok(
        "A password reset email will be sent if the email address is valid.".to_string(),
    ))
}

/// Use a token provided in an email to reset a user's password.
#[post("/reset_password")]
#[tracing::instrument(skip(params, db))]
async fn reset_password(
    params: web::Json<ResetPasswordParams>,
    db: Data<Repo>,
) -> Result<HttpResponse> {
    let token_clone = params.token.clone();

    if let Err(e) = validate_password_strength(&params.new_password) {
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
