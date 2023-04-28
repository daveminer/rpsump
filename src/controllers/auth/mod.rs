use actix_web::web::ServiceConfig;
use serde::{Deserialize, Serialize};

pub mod login;
pub mod logout;
pub mod reset_password;
pub mod signup;

#[derive(Clone, Serialize, Deserialize)]
pub struct AuthParams {
    email: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
pub struct SignupParams {
    email: String,
    password: String,
    confirm_password: String,
}

pub fn auth_routes(cfg: &mut ServiceConfig) {
    cfg.service(signup::signup);
    cfg.service(login::login);
    cfg.service(logout::logout);
    cfg.service(reset_password::reset_password);
}
