use actix_web::web::ServiceConfig;
use secrecy::Secret;
use serde::Deserialize;

pub mod email_verification;
pub mod login;
pub mod logout;
pub mod reset_password;
pub mod signup;

#[derive(Deserialize)]
pub struct AuthParams {
    email: String,
    password: Secret<String>,
}

#[derive(Deserialize)]
pub struct SignupParams {
    email: String,
    password: Secret<String>,
    confirm_password: Secret<String>,
}

pub fn auth_routes(cfg: &mut ServiceConfig) {
    cfg.service(email_verification::verify_email);
    cfg.service(login::login);
    cfg.service(logout::logout);
    cfg.service(reset_password::reset_password);
    cfg.service(signup::signup);
}
