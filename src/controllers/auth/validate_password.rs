use validator::ValidationError;

use crate::{
    auth::password::Password,
    util::{
        PASSWORD_LOWER, PASSWORD_NUMBER, PASSWORD_SPECIAL, PASSWORD_TOO_LONG, PASSWORD_TOO_SHORT,
        PASSWORD_UPPER,
    },
};

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
