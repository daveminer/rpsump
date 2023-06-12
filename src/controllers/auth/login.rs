use actix_identity::Identity;
use actix_web::{post, web, web::Data, HttpMessage, HttpRequest, HttpResponse, Responder, Result};
use anyhow::{anyhow, Error};
use bcrypt::verify;
use diesel::prelude::*;
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};

use crate::auth::{claim::create_token, validate_credentials, AuthParams};
use crate::config::Settings;
use crate::controllers::ApiResponse;
use crate::database::DbPool;
use crate::middleware::telemetry::spawn_blocking_with_tracing;
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

    // Log user in
    Identity::login(&request.extensions(), user.id.to_string()).expect("Could not log identity in");

    // Create user event
    let mut conn = db.get().expect("Could not get a db connection.");
    let _user_event = conn.transaction::<_, Error, _>(|conn| {
        let user_event: UserEvent = diesel::insert_into(user_event::table)
            .values((
                user_event::user_id.eq(user.id),
                user_event::event_type.eq(EventType::Login.to_string()),
                user_event::ip_address.eq(super::ip_address(&request)),
            ))
            .get_result(conn)?;

        Ok(user_event)
    });

    // Create token
    let token = create_token(user.id, settings.jwt_secret.clone())
        .expect("Could not create token for user.");

    Ok(HttpResponse::Ok().json(Response { token }))
}
