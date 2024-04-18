use actix_web::{HttpRequest, HttpResponse};
use anyhow::{anyhow, Error};
use secrecy::ExposeSecret;
use validator::ValidationError;

use crate::{
    auth::password::Password,
    util::{
        ApiResponse, PASSWORD_LOWER, PASSWORD_NUMBER, PASSWORD_SPECIAL, PASSWORD_TOO_LONG,
        PASSWORD_TOO_SHORT, PASSWORD_UPPER,
    },
};

pub fn error_response(e: Error, msg: &str) -> HttpResponse {
    tracing::error!(target = module_path!(), error = e.to_string(), msg);
    ApiResponse::internal_server_error()
}

pub fn invalid_token_response() -> HttpResponse {
    ApiResponse::bad_request("Invalid token.".to_string())
}

pub fn ip_address(req: &HttpRequest) -> Result<String, anyhow::Error> {
    match req.connection_info().peer_addr() {
        Some(ip) => Ok(ip.to_string()),
        None => Err(anyhow!("Could not get IP address from request.")),
    }
}

// Ensure passwords adhere to safety standards
#[tracing::instrument(skip(password))]
pub fn validate_password_strength(password: &Password) -> Result<(), ValidationError> {
    let secret = password.expose_secret();

    if secret.len() < 8 {
        return Err(ValidationError::new(PASSWORD_TOO_SHORT));
    }

    if secret.len() > 72 {
        return Err(ValidationError::new(PASSWORD_TOO_LONG));
    }

    if !secret.chars().any(char::is_uppercase) {
        return Err(ValidationError::new(PASSWORD_UPPER));
    }

    if !secret.chars().any(char::is_lowercase) {
        return Err(ValidationError::new(PASSWORD_LOWER));
    }

    if !secret.chars().any(char::is_numeric) {
        return Err(ValidationError::new(PASSWORD_NUMBER));
    }

    if !secret.chars().any(|c| !c.is_alphanumeric()) {
        return Err(ValidationError::new(PASSWORD_SPECIAL));
    }

    Ok(())
}
