use std::sync::Arc;

use actix_web::{post, web, web::Data, HttpRequest, HttpResponse, Result};
use serde::Deserialize;
use validator::Validate;

use crate::auth::password::Password;
use crate::config::Settings;
use crate::controllers::auth::helpers::{error_response, ip_address, validate_password_strength};
use crate::repository::Repo;
use crate::util::ApiResponse;

#[derive(Debug, Deserialize, Validate)]
pub struct SignupParams {
    #[validate(email)]
    email: String,
    #[validate(custom = "validate_password_strength")]
    password: Password,
    #[validate(must_match(
        other = "password",
        message = "Password and confirm password must match."
    ))]
    confirm_password: Password,
}

#[post("/signup")]
#[tracing::instrument(skip(params, repo, settings))]
pub async fn signup(
    req: HttpRequest,
    params: web::Json<SignupParams>,
    repo: Data<Repo>,
    settings: Data<Settings>,
) -> Result<HttpResponse> {
    // Validate params
    match &params.validate() {
        Ok(_) => (),
        Err(e) => return Ok(ApiResponse::bad_request(e.to_string())),
    };

    let ip_addr: String = match ip_address(&req) {
        Ok(ip) => ip,
        Err(e) => return Ok(error_response(e, "User signup failed")),
    };

    // Hash password
    let hash = match params.password.hash() {
        Ok(password_hash) => password_hash,
        Err(_) => {
            return Ok(ApiResponse::bad_request(
                "Try a different password.".to_string(),
            ))
        }
    };

    // Create user
    let new_user = match repo.create_user(params.email.clone(), hash, ip_addr).await {
        Ok(user) => user,
        Err(e) => return Ok(ApiResponse::bad_request(e.to_string())),
    };

    let mailer_settings = Arc::clone(&settings).mailer.clone();

    // Send email verification
    match new_user
        .send_email_verification(mailer_settings, req.connection_info().host())
        .await
    {
        Ok(_) => Ok(ApiResponse::ok("User created.".to_string())),
        Err(e) => Ok(error_response(e, "Email verification failed")),
    }
}
