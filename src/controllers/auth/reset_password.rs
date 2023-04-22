use actix_identity::Identity;
use actix_web::{post, web, web::Data, HttpResponse, Responder, Result};
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};

use crate::auth::claim::Claim;
use crate::controllers::auth::TOKEN_EXPIRATION_TIME_SECONDS;
use crate::database::DbPool;

#[post("/reset_password")]
async fn reset_password(
    identity: Identity,
    email: web::Json<String>,
    db: Data<DbPool>,
) -> Result<impl Responder> {
    let user_email = email.into_inner();

    let token = encode(
        &Header::default(),
        &Claim {
            sub: identity
                .id()
                .expect("Could not get id of identity")
                .to_string(),
            exp: Utc::now()
                .checked_add_signed(chrono::Duration::seconds(
                    TOKEN_EXPIRATION_TIME_SECONDS as i64,
                ))
                .expect("valid timestamp")
                .timestamp() as u64,
            iat: Utc::now().timestamp_millis() as u64,
        },
        &EncodingKey::from_secret(&[0; 32]),
    )
    .expect("Could not encode token");
    // TODO: generate a new password reset token and save it to the database or cache
    Ok(HttpResponse::Ok().finish())
}
