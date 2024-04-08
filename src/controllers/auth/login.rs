use actix_web::{post, web, web::Data, HttpRequest, HttpResponse, Result};
use anyhow::{anyhow, Error};
use bcrypt::verify;
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};

use crate::auth::{claim::create_token, password::AuthParams};
use crate::config::Settings;
use crate::controllers::auth::helpers::{error_response, ip_address};
use crate::repository::models::user::UserFilter;
use crate::repository::{
    models::{user::User, user_event::*},
    Repo,
};
use crate::util::{spawn_blocking_with_tracing, ApiResponse, BAD_CREDS, REQUIRED_FIELDS};

#[derive(Serialize, Deserialize)]
struct Response {
    token: String,
}

#[post("/login")]
#[tracing::instrument(skip(request, user_data, repo, settings))]
pub async fn login(
    request: HttpRequest,
    user_data: web::Json<AuthParams>,
    settings: Data<Settings>,
    repo: Data<Repo>,
) -> Result<HttpResponse> {
    // User lookup from params
    let AuthParams { email, password } = user_data.into_inner();
    if email.is_empty() || password.expose_secret().is_empty() {
        return Ok(ApiResponse::bad_request(REQUIRED_FIELDS.to_string()));
    }

    let user_filter = UserFilter {
        email: Some(email),
        ..Default::default()
    };

    let maybe_user = match repo.users(user_filter).await {
        Ok(users) => match users.len() {
            0 => None,
            1 => users.first().cloned(),
            _ => {
                return Ok(error_response(
                    anyhow!("duplicate_email"),
                    "More than one record found for email.",
                ))
            }
        },
        Err(e) => {
            return Ok(ApiResponse::bad_request(e.to_string()));
        }
    };

    if let Err(e) = verify_password(maybe_user.clone(), password).await {
        return Ok(ApiResponse::bad_request(e.to_string()));
    };

    let ip_addr: String = match ip_address(&request) {
        Ok(ip) => ip,
        Err(e) => return Ok(error_response(e, "User signup failed")),
    };

    let user = maybe_user.unwrap();
    // Create token
    let token = match create_token(
        user.id,
        settings.jwt_secret.clone(),
        settings.server.token_duration_days,
    ) {
        Ok(token) => token,
        Err(e) => return Ok(error_response(e, "Could not create token for user")),
    };

    let _user_event = match repo
        .create_user_event(&user, EventType::Login, ip_addr)
        .await
    {
        Ok(user_event) => user_event,
        Err(e) => return Ok(error_response(e, "Could not create user event")),
    };

    tracing::info!(target = module_path!(), user_id = user.id, "User logged in");

    Ok(HttpResponse::Ok().json(Response { token }))
}

#[tracing::instrument(skip(user, password))]
async fn verify_password(user: Option<User>, password: Secret<String>) -> Result<(), Error> {
    let provided_pw: String = match user {
        Some(user) => user.password_hash,
        None => "decoy".to_string(),
    };

    // Check if password is correct
    spawn_blocking_with_tracing(move || {
        match verify(password.expose_secret(), &provided_pw) {
            Ok(true) => Ok(()),
            // Match on false and Err
            _ => Err(anyhow!(BAD_CREDS.to_string())),
        }
    })
    .await?
}
