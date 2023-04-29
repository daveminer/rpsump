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
