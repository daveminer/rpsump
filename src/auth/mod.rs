use anyhow::{anyhow, Error};
use bcrypt::{hash, DEFAULT_COST};

pub mod authenticated_user;
pub mod claim;
pub mod token;

pub fn hash_user_password(password: &str) -> Result<String, Error> {
    hash(password, DEFAULT_COST).map_err(|_| anyhow!("Could not hash password."))
}

// Ensure passwords adhere to safety standards
pub fn validate_password(password: &String, password_confirmation: &String) -> Result<(), Error> {
    validate_password_length(password)?;

    if !password.chars().any(char::is_uppercase) {
        return Err(anyhow!("Password must have at least one uppercase letter."));
    }

    if !password.chars().any(char::is_lowercase) {
        return Err(anyhow!("Password must have at least one lowercase letter."));
    }

    if !password.chars().any(char::is_numeric) {
        return Err(anyhow!("Password must have at least one number."));
    }

    if !password.chars().any(|c| !c.is_alphanumeric()) {
        return Err(anyhow!(
            "Password must have at least one special character."
        ));
    }

    if password != password_confirmation {
        return Err(anyhow!("Passwords do not match."));
    }

    Ok(())
}

pub fn validate_password_length(password: &String) -> Result<(), Error> {
    if password.len() < 8 {
        return Err(anyhow!("Password is too short."));
    }

    if password.len() > 72 {
        return Err(anyhow!("Password is too long."));
    }

    Ok(())
}
