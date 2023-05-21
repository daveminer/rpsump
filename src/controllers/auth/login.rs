use actix_identity::Identity;
use actix_web::{post, web, web::Data, HttpMessage, HttpRequest, HttpResponse, Responder, Result};
use anyhow::{anyhow, Error};
use bcrypt::verify;
use diesel::prelude::*;
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};

use crate::auth::claim::create_token;
use crate::auth::{validate_credentials, AuthParams};
use crate::config::Settings;
use crate::controllers::ErrorBody;
use crate::database::DbPool;
use crate::middleware::telemetry::spawn_blocking_with_tracing;
use crate::models::user_event::*;
use crate::schema::user_event;

const BAD_CREDS: &str = "Invalid email or password.";

#[derive(Serialize, Deserialize)]
struct Response {
    token: String,
}

#[post("/login")]
#[tracing::instrument(
    skip(request, db, user_data, settings),
    fields(email=tracing::field::Empty, password=tracing::field::Empty)
)]
pub async fn login(
    request: HttpRequest,
    user_data: web::Json<AuthParams>,
    db: Data<DbPool>,
    settings: Data<Settings>,
) -> Result<impl Responder> {
    // User lookup from params
    let credentials: AuthParams = user_data.into_inner();
    let user = match validate_credentials(&credentials, db.clone()).await {
        Ok(user) => user,
        Err(e) => {
            return Ok(HttpResponse::BadRequest().json(ErrorBody {
                reason: e.to_string(),
            }))
        }
    };

    // Check if user is allowed to login
    let ip_addr = super::ip_address(&request);
    if let Err(e) = UserEvent::check_allowed_status(
        Some(user.clone()),
        ip_addr.to_string(),
        settings.auth_attempts_allowed,
        db.clone(),
    )
    .await
    {
        return Ok(HttpResponse::Unauthorized().json(ErrorBody {
            reason: e.to_string(),
        }));
    }

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
            return Ok(HttpResponse::BadRequest().json(ErrorBody {
                reason: e.to_string(),
            }));
        }
    };

    // Log user in
    Identity::login(&request.extensions(), user.id.to_string()).expect("Could not log identity in");

    // Create user event
    let mut conn = db.get().expect("Could not get a db connection.");
    let _user_event = conn.transaction::<_, Error, _>(|conn| {
        let user_event: UserEvent = diesel::insert_into(user_event::table)
            .values((
                user_event::user_id.eq(user.id),
                user_event::event_type.eq(EventType::Login.to_string()),
                user_event::ip_address.eq(ip_addr.clone()),
            ))
            .get_result(conn)?;

        Ok(user_event)
    });

    // Create token
    let token = create_token(user.id, settings.jwt_secret.clone())
        .expect("Could not create token for user.");

    Ok(HttpResponse::Ok().json(Response { token }))
}
