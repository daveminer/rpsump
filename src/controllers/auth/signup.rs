use actix_web::{post, web, web::Data, HttpRequest, Responder, Result};
use serde::Deserialize;
use validator::Validate;

use crate::auth::{password::Password, validate_password};
use crate::config::Settings;
use crate::controllers::{auth::ip_address, ApiResponse};
use crate::database::DbPool;
use crate::models::user::User;

#[derive(Debug, Deserialize, Validate)]
pub struct SignupParams {
    #[validate(email)]
    email: String,
    #[validate(custom = "validate_password")]
    password: Password,
    #[validate(must_match(
        other = "password",
        message = "Password and confirm password must match."
    ))]
    confirm_password: Password,
}

#[post("/signup")]
#[tracing::instrument(skip(params, db, settings))]
pub async fn signup(
    req: HttpRequest,
    params: web::Json<SignupParams>,
    db: Data<DbPool>,
    settings: Data<Settings>,
) -> Result<impl Responder> {
    // Validate params
    match &params.validate() {
        Ok(_) => (),
        Err(e) => return Ok(ApiResponse::bad_request(e.to_string())),
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

    let ip_addr: String = match ip_address(&req) {
        Ok(ip) => ip,
        Err(e) => {
            tracing::error!("User signup failed: {}", e);
            return Ok(ApiResponse::internal_server_error());
        }
    };

    // Create user
    let new_user = match User::create(params.email.to_string(), hash, ip_addr, db.clone()).await {
        Ok(user) => user,
        Err(e) => {
            return Ok(ApiResponse::bad_request(e.to_string()));
        }
    };

    // Send email verification
    match new_user
        .send_email_verification(
            db.clone(),
            &settings.mailer.server_url,
            req.connection_info().host(),
            &settings.mailer.auth_token.clone(),
        )
        .await
    {
        Ok(_) => Ok(ApiResponse::ok("User created.".to_string())),
        Err(_e) => Ok(ApiResponse::internal_server_error()),
    }
}
