use actix_web::{post, web, web::Data, HttpRequest, HttpResponse, Responder, Result};
use anyhow::{anyhow, Error};
use bcrypt::verify;
use diesel::prelude::*;
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};

use crate::auth::claim::create_token;
use crate::auth::password::AuthParams;
use crate::config::Settings;
use crate::controllers::auth::ip_address;
use crate::database::{DbPool, RealDbPool};
use crate::models::user::User;
use crate::models::user_event::*;
use crate::new_conn;
use crate::schema::user_event;
use crate::util::{spawn_blocking_with_tracing, ApiResponse};

const BAD_CREDS: &str = "Invalid email or password.";
pub const REQUIRED_FIELDS: &str = "Email and password are required.";

#[derive(Serialize, Deserialize)]
struct Response {
    token: String,
}

#[post("/login")]
#[tracing::instrument(skip(request, db, user_data, settings))]
pub async fn login(
    request: HttpRequest,
    user_data: web::Json<AuthParams>,
    db: Data<RealDbPool>,
    settings: Data<Settings>,
) -> Result<impl Responder> {
    let mut conn = new_conn!(db);

    // User lookup from params
    let credentials: AuthParams = user_data.into_inner();
    let user = match validate_credentials(&credentials, db).await {
        Ok(user) => user,
        Err(e) => return Ok(ApiResponse::bad_request(e.to_string())),
    };

    // Check if password is correct
    match spawn_blocking_with_tracing(move || {
        // Resist timing attacks by always hashing the password
        match verify(
            credentials.password.expose_secret(),
            user.password_hash.as_str(),
        ) {
            Ok(true) => Ok(()),
            // Match on false and Err
            _ => Err(anyhow!(BAD_CREDS.to_string())),
        }
    })
    .await
    .unwrap()
    {
        Ok(()) => (),
        Err(e) => {
            return Ok(ApiResponse::bad_request(e.to_string()));
        }
    };

    let ip_addr: String = match ip_address(&request) {
        Ok(ip) => ip,
        Err(e) => {
            tracing::error!(
                target = module_path!(),
                error = e.to_string(),
                "User signup failed"
            );
            return Ok(ApiResponse::internal_server_error());
        }
    };

    // Create user event
    let user_event = spawn_blocking_with_tracing(move || {
        return conn.transaction::<_, Error, _>(|conn| {
            diesel::insert_into(user_event::table)
                .values((
                    user_event::user_id.eq(user.id),
                    user_event::event_type.eq(EventType::Login.to_string()),
                    user_event::ip_address.eq(ip_addr),
                ))
                .execute(conn)?;

            Ok(())
        });
    });

    match user_event.await {
        Ok(_) => (),
        Err(e) => {
            tracing::error!(
                target = module_path!(),
                error = e.to_string(),
                "Could not insert user event during signup"
            );
            return Ok(ApiResponse::internal_server_error());
        }
    };

    // Create token
    let token = match create_token(user.id, settings.jwt_secret.clone()) {
        Ok(token) => token,
        Err(e) => {
            tracing::error!(
                target = module_path!(),
                error = e.to_string(),
                "Could not create token for user"
            );
            return Ok(ApiResponse::internal_server_error());
        }
    };

    tracing::info!(target = module_path!(), user_id = user.id, "User logged in");

    Ok(HttpResponse::Ok().json(Response { token }))
}

#[tracing::instrument(skip(credentials, db))]
async fn validate_credentials(
    credentials: &AuthParams,
    db: Data<RealDbPool>,
) -> Result<User, Error> {
    // User lookup from params
    let AuthParams { email, password } = credentials;
    if email.is_empty() || password.expose_secret().is_empty() {
        return Err(anyhow!(REQUIRED_FIELDS.to_string()));
    }

    let email_clone = email.clone();

    // TODO: remove unwrap
    let user_query = spawn_blocking_with_tracing(move || {
        User::by_email(email_clone.clone()).first::<User>(&mut db.get_conn().unwrap())
    })
    .await?;

    let user = match user_query {
        Ok(user) => user,
        Err(_not_found) => {
            return Err(anyhow!(BAD_CREDS.to_string()));
        }
    };

    Ok(user)
}
