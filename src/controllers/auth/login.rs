use actix_identity::Identity;
use actix_web::error;
use actix_web::{post, web, web::Data, HttpMessage, HttpRequest, HttpResponse, Responder, Result};
use bcrypt::verify;
use diesel::prelude::*;

use crate::auth::claim::create_token;
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

    let user: User =
        first!(User::by_email(email), User, db).or(Err(error::ErrorUnauthorized(BAD_CREDS)))?;

    verify(password, &user.password_hash)
        .or(Err(error::ErrorUnauthorized(BAD_CREDS)))
        .expect("Invalid password.");

    Identity::login(&request.extensions(), user.id.to_string()).expect("Could not log identity in");

    let token = create_token(user.id, settings.jwt_secret.clone())
        .expect("Could not create token for user.");

    Ok(HttpResponse::Ok().body(token))
}
