use actix_web::{web, web::Data};
use anyhow::{anyhow, Error};
use diesel::RunQueryDsl;
use secrecy::{ExposeSecret, Secret};
use validator::ValidationError;

use crate::auth::password::Password;
use crate::database::{first, DbPool};
use crate::models::user::User;

pub mod authenticated_user;
pub mod claim;
pub mod password;
pub mod token;

pub const BAD_CREDS: &str = "Invalid email or password.";
pub const REQUIRED_FIELDS: &str = "Email and password are required.";
const PASSWORD_UPPER: &str = "Password must contain an uppercase letter.";
const PASSWORD_LOWER: &str = "Password must contain a lowercase letter.";
const PASSWORD_NUMBER: &str = "Password must contain a number.";
const PASSWORD_SPECIAL: &str = "Password must contain a special character.";
const PASSWORD_TOO_SHORT: &str = "Password is too short.";
const PASSWORD_TOO_LONG: &str = "Password is too long.";

#[derive(Debug, serde::Deserialize)]
pub struct AuthParams {
    pub email: String,
    pub password: Secret<String>,
}

#[tracing::instrument(skip(credentials, pool))]
pub async fn validate_credentials(
    credentials: &AuthParams,
    pool: Data<DbPool>,
) -> Result<User, Error> {
    // User lookup from params
    let AuthParams { email, password } = credentials;
    if email.is_empty() || password.expose_secret().is_empty() {
        return Err(anyhow!(REQUIRED_FIELDS.to_string()));
    }

    let email_clone = email.clone();

    let user = match first!(User::by_email(email_clone), User, pool.clone()) {
        Ok(user) => user,
        Err(_not_found) => {
            return Err(anyhow!(BAD_CREDS.to_string()));
        }
    };

    Ok(user)
}

// Ensure passwords adhere to safety standards
#[tracing::instrument(skip(password))]
pub fn validate_password(password: &Password) -> Result<(), ValidationError> {
    let secret = password.expose_secret();
    validate_password_length(password)?;

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

#[tracing::instrument(skip(password))]
pub fn validate_password_length(password: &Password) -> Result<(), ValidationError> {
    let secret = password.expose_secret();

    if secret.len() < 8 {
        return Err(ValidationError::new(PASSWORD_TOO_SHORT));
    }

    if secret.len() > 72 {
        return Err(ValidationError::new(PASSWORD_TOO_LONG));
    }

    Ok(())
}
