use actix_web::{post, web, web::Data, HttpRequest, HttpResponse, Result};
use anyhow::anyhow;
use serde::Deserialize;
use validator::Validate;

use crate::auth::password::Password;
use crate::config::Settings;
use crate::controllers::auth::helpers::{error_response, ip_address, validate_password_strength};
use crate::email::sendinblue::send_password_reset;
use crate::repository::models::user::UserFilter;
use crate::repository::Repo;
use crate::util::ApiResponse;

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
// TODO: combine db calls
#[post("/request_password_reset")]
#[tracing::instrument(skip(req, params, repo, settings))]
async fn request_password_reset(
    req: HttpRequest,
    params: web::Json<RequestPasswordResetParams>,
    repo: Data<Repo>,
    settings: Data<Settings>,
) -> Result<HttpResponse> {
    let _ip_addr: String = match ip_address(&req) {
        Ok(ip) => ip,
        Err(e) => return Ok(error_response(e, "Getting ip address from request failed")),
    };

    let filter = UserFilter {
        email: Some(params.email.clone()),
        ..Default::default()
    };

    let users = match repo.users(filter).await {
        Ok(users) => users,
        Err(e) => return Ok(error_response(e, "User lookup failed")),
    };

    if users.len() > 1 {
        return Ok(error_response(
            anyhow!("Duplicate email."),
            "Could not save password reset token.",
        ));
    }

    if users.len() == 1 {
        let mut user = users.first().unwrap().clone();

        if let Some(token) = user.password_reset_token_expires_at.clone() {
            if token < chrono::Utc::now().naive_utc() {
                return Ok(ApiResponse::bad_request(
                    "Password reset request expired; please try again.".to_string(),
                ));
            }
        }

        let token = match repo.create_password_reset(user.clone()).await {
            Ok(token) => token,
            Err(e) => return Ok(error_response(e, "Could not save password reset token")),
        };
        let auth_token = &settings.mailer.auth_token.clone();

        user.password_reset_token = Some(token.value);
        let _ = send_password_reset(
            user,
            &settings.mailer.server_url,
            req.connection_info().host(),
            auth_token,
        )
        .await;
    }

    // TODO: write event with ip_addr

    Ok(ApiResponse::ok(
        "A password reset email will be sent if the email address is valid.".to_string(),
    ))
}

/// Use a token provided in an email to reset a user's password.
#[post("/reset_password")]
#[tracing::instrument(skip(params, repo))]
async fn reset_password(
    params: web::Json<ResetPasswordParams>,
    repo: Data<Repo>,
) -> Result<HttpResponse> {
    let password = &params.new_password;
    let token = &params.token;

    if let Err(e) = validate_password_strength(password) {
        return Ok(ApiResponse::bad_request(e.to_string()));
    };

    // TODO: type these errors
    match repo.reset_password(password, token.clone()).await {
        Ok(_) => Ok(ApiResponse::ok("Password reset successfully.".to_string())),
        Err(e) => Ok(ApiResponse::bad_request(e.to_string())),
    }
}
