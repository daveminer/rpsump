use actix_web::web::ServiceConfig;

pub mod email_verification;
pub mod helpers;
pub mod login;
pub mod reset_password;
pub mod signup;

pub fn auth_routes(cfg: &mut ServiceConfig) {
    cfg.service(email_verification::verify_email);
    cfg.service(login::login);
    cfg.service(reset_password::reset_password);
    cfg.service(reset_password::request_password_reset);
    cfg.service(signup::signup);
}
