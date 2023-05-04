use actix_identity::Identity;
use actix_web::error;
use actix_web::{post, web, web::Data, HttpMessage, HttpRequest, HttpResponse, Responder, Result};
use bcrypt::verify;
use diesel::prelude::*;

use crate::auth::claim::create_token;
use crate::auth::validate_password_length;
use crate::controllers::auth::AuthParams;
use crate::database::{first, DbPool};
use crate::Settings;

use crate::models::user::User;

const BAD_CREDS: &str = "Invalid email or password";

#[post("/login")]
async fn login(
    request: HttpRequest,
    user_data: web::Json<AuthParams>,
    db: Data<DbPool>,
    settings: Data<Settings>,
) -> Result<impl Responder> {
    let AuthParams { email, password } = user_data.into_inner();

    validate_password_length(&password).map_err(|e| error::ErrorBadRequest(e))?;

    // Resist timing attacks by always hashing the password
    let (user, existing_user_password_hash) = match first!(User::by_email(email), User, db) {
        Ok(user) => (Some(user.clone()), user.password_hash),
        Err(_e) => (None, "".to_string()),
    };

    verify(password, &existing_user_password_hash)
        .or(Err(error::ErrorUnauthorized(BAD_CREDS)))
        .expect("Invalid user or password.");

    let user_id = user.unwrap().id;

    Identity::login(&request.extensions(), user_id.to_string()).expect("Could not log identity in");

    let token = create_token(user_id, settings.jwt_secret.clone())
        .expect("Could not create token for user.");

    Ok(HttpResponse::Ok().body(token))
}
