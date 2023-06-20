use actix_web::{post, web, web::Data, HttpRequest, HttpResponse, Responder, Result};
use anyhow::{anyhow, Error};
use bcrypt::verify;
use diesel::prelude::*;
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};

use super::super::spawn_blocking_with_tracing;
use crate::auth::{claim::create_token, validate_credentials, AuthParams};
use crate::config::Settings;
use crate::controllers::{auth::ip_address, ApiResponse};
use crate::database::DbPool;
use crate::models::user_event::*;
use crate::new_conn;
use crate::schema::user_event;

const BAD_CREDS: &str = "Invalid email or password.";

#[derive(Serialize, Deserialize)]
struct Response {
    token: String,
}

#[post("/login")]
#[tracing::instrument(skip(request, db, user_data, settings))]
pub async fn login(
    request: HttpRequest,
    user_data: web::Json<AuthParams>,
    db: Data<DbPool>,
    settings: Data<Settings>,
) -> Result<impl Responder> {
    let conn = new_conn!(db);

    // User lookup from params
    let credentials: AuthParams = user_data.into_inner();
    let user = match validate_credentials(&credentials, conn).await {
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
            tracing::error!("User signup failed: {}", e);
            return Ok(ApiResponse::internal_server_error());
        }
    };

    // Create user event
    let mut conn = new_conn!(db);
    let user_event = conn.transaction::<_, Error, _>(|conn| {
        diesel::insert_into(user_event::table)
            .values((
                user_event::user_id.eq(user.id),
                user_event::event_type.eq(EventType::Login.to_string()),
                user_event::ip_address.eq(ip_addr),
            ))
            .execute(conn)?;

        Ok(())
    });

    match user_event {
        Ok(_) => (),
        Err(e) => {
            tracing::error!("Could not insert user event during signup: {}", e);
            return Ok(ApiResponse::internal_server_error());
        }
    };

    // Create token
    let token = match create_token(user.id, settings.jwt_secret.clone()) {
        Ok(token) => token,
        Err(e) => {
            tracing::error!("Could not create token for user: {}", e);
            return Ok(ApiResponse::internal_server_error());
        }
    };

    tracing::info!("User logged in: {}", user.id);

    Ok(HttpResponse::Ok().json(Response { token }))
}
