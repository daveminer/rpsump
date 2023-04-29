use actix_web::web::ServiceConfig;
use anyhow::{anyhow, Error};
use serde::{Deserialize, Serialize};

pub mod email_verification;
pub mod login;
pub mod logout;
pub mod reset_password;
pub mod signup;

#[derive(Clone, Serialize, Deserialize)]
pub struct AuthParams {
    email: String,
    password: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SignupParams {
    email: String,
    password: String,
    confirm_password: String,
}

pub fn auth_routes(cfg: &mut ServiceConfig) {
    cfg.service(email_verification::verify_email);
    cfg.service(login::login);
    cfg.service(logout::logout);
    cfg.service(reset_password::reset_password);
    cfg.service(signup::signup);
}

// Ensure passwords adhere to safety standards
pub fn validate_password(password: &String, password_confirmation: &String) -> Result<(), Error> {
    if password.len() < 8 {
        return Err(anyhow!("Password is too short."));
    }

    if password.len() > 72 {
        return Err(anyhow!("Password is too long."));
    }

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
