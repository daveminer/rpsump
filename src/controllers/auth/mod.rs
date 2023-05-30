use actix_web::web::ServiceConfig;
use actix_web::HttpRequest;

pub mod email_verification;
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

pub fn ip_address(req: &HttpRequest) -> String {
    req.connection_info()
        .peer_addr()
        .expect("Could not get IP address.")
        .to_string()
}
