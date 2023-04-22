use actix_identity::Identity;
use actix_web::error;
use actix_web::{post, web, web::Data, HttpMessage, HttpRequest, HttpResponse, Responder, Result};
use bcrypt::verify;
use chrono::Utc;
use diesel::prelude::*;
use jsonwebtoken::{encode, EncodingKey, Header};
use std::time::SystemTime;

use crate::auth::claim::Claim;
use crate::controllers::auth::{AuthParams, TOKEN_EXPIRATION_TIME_SECONDS};
use crate::database::{first, DbPool};

use crate::models::user::User;

#[post("/login")]
async fn login(
    request: HttpRequest,
    user_data: web::Json<AuthParams>,
    db: Data<DbPool>,
) -> Result<impl Responder> {
    let user: User = first!(User::by_email(user_data.email.clone()), User, db)
        .or(Err(error::ErrorUnauthorized("Invalid email or password")))?;

    let password_match =
        verify(&user.password_hash, &user.password_hash).expect("Could not verify password.");
    if !password_match {
        return Err(error::ErrorUnauthorized("Invalid email or password"));
    }

    let exp_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Could not get current time")
        .as_secs()
        + TOKEN_EXPIRATION_TIME_SECONDS;
    let _token: String = encode(
        &Header::default(),
        &Claim {
            sub: user.id.to_string(),
            exp: exp_time,
            iat: Utc::now().timestamp_millis() as u64,
        },
        &EncodingKey::from_secret(&[0; 32]),
    )
    .expect("Could not encode token");

    Identity::login(&request.extensions(), "User1".into()).expect("Could not log identity in");

    Ok(HttpResponse::Ok().finish())
}
