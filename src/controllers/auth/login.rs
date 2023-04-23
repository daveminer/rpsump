use actix_identity::Identity;
use actix_web::error;
use actix_web::{post, web, web::Data, HttpMessage, HttpRequest, HttpResponse, Responder, Result};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use diesel::prelude::*;
use jsonwebtoken::{encode, EncodingKey, Header};
use std::time::SystemTime;

use crate::auth::claim::Claim;
use crate::controllers::auth::{AuthParams, TOKEN_EXPIRATION_TIME_SECONDS};
use crate::database::{first, DbPool};

use crate::models::user::User;

const BAD_CREDS: &str = "Invalid email or password";

#[post("/login")]
async fn login(
    request: HttpRequest,
    user_data: web::Json<AuthParams>,
    db: Data<DbPool>,
) -> Result<impl Responder> {
    let AuthParams { email, password } = user_data.into_inner();

    let user: User =
        first!(User::by_email(email), User, db).or(Err(error::ErrorUnauthorized(BAD_CREDS)))?;

    verify(password, &user.password_hash)
        .or(Err(error::ErrorUnauthorized(BAD_CREDS)))
        .expect("Invalid password.");

    let exp_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Could not get current time")
        .as_secs()
        + TOKEN_EXPIRATION_TIME_SECONDS;
    let token: String = encode(
        &Header::default(),
        &Claim {
            sub: user.id.to_string(),
            exp: exp_time,
            iat: Utc::now().timestamp_millis() as u64,
        },
        &EncodingKey::from_secret("secret".as_ref()),
    )
    .expect("Could not encode token");

    Identity::login(&request.extensions(), "User1".into()).expect("Could not log identity in");

    Ok(HttpResponse::Ok().body(token))
}
