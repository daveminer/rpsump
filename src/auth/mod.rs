use anyhow::{anyhow, Error};
use bcrypt::{hash, DEFAULT_COST};
use secrecy::{ExposeSecret, Secret};

pub mod authenticated_user;
pub mod claim;
pub mod token;

pub fn hash_user_password(password: &str) -> Result<String, Error> {
    hash(password, DEFAULT_COST).map_err(|_| anyhow!("Could not hash password."))
}

// Ensure passwords adhere to safety standards
pub fn validate_password(
    password: &Secret<String>,
    password_confirmation: &Secret<String>,
) -> Result<(), Error> {
    let secret = password.expose_secret();
    validate_password_length(password)?;

    if !secret.chars().any(char::is_uppercase) {
        return Err(anyhow!("Password must have at least one uppercase letter."));
    }

    if !secret.chars().any(char::is_lowercase) {
        return Err(anyhow!("Password must have at least one lowercase letter."));
    }

    if !secret.chars().any(char::is_numeric) {
        return Err(anyhow!("Password must have at least one number."));
    }

    if !secret.chars().any(|c| !c.is_alphanumeric()) {
        return Err(anyhow!(
            "Password must have at least one special character."
        ));
    }

    if secret != password_confirmation.expose_secret() {
        return Err(anyhow!("Passwords do not match."));
    }

    Ok(())
}

pub fn validate_password_length(password: &Secret<String>) -> Result<(), Error> {
    let secret = password.expose_secret();

    if secret.len() < 8 {
        return Err(anyhow!("Password is too short."));
    }

    if secret.len() > 72 {
        return Err(anyhow!("Password is too long."));
    }

    Ok(())
}
