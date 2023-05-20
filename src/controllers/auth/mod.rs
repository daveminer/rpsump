use actix_web::{web, web::Data, web::ServiceConfig};
use actix_web::{HttpRequest, HttpResponse};
use anyhow::{anyhow, Error};
use diesel::RunQueryDsl;
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;

use crate::database::{first, DbPool};
use crate::models::user::User;
use crate::models::user_event::UserEvent;

pub mod email_verification;
pub mod login;
pub mod logout;
pub mod reset_password;
pub mod signup;

const BAD_CREDS: &str = "Invalid email or password.";
const REQUIRED_FIELDS: &str = "Email and password are required.";

#[derive(Debug, Deserialize)]
pub struct AuthParams {
    email: String,
    password: Secret<String>,
}

#[derive(Debug, Deserialize)]
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

#[tracing::instrument(name = "Validate credentials", skip(credentials, pool))]
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

pub fn ip_address(req: &HttpRequest) -> String {
    req.connection_info()
        .peer_addr()
        .expect("Could not get IP address.")
        .to_string()
}
