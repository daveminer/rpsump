use actix_web::{post, web, web::Data, HttpRequest, HttpResponse, Responder, Result};
use bcrypt::{hash, DEFAULT_COST};
use diesel::prelude::*;

use crate::controllers::auth::SignupParams;
use crate::database::{first, DbPool};
use crate::models::user::User;
use crate::Settings;

#[post("/signup")]
pub async fn signup(
    req: HttpRequest,
    user_data: web::Json<SignupParams>,
    db: Data<DbPool>,
    settings: Data<Settings>,
) -> Result<impl Responder> {
    let new_user = user_data.into_inner();
    let db_clone = db.clone();
    let new_user_clone = new_user.clone();

    if new_user.password.len() < 8 {
        return Ok(HttpResponse::BadRequest().body("Password must be at least 8 characters."));
    }

    if new_user.password != new_user.confirm_password {
        return Ok(HttpResponse::BadRequest().body("Password and confirmation do not match."));
    }

    if let Ok(_user) = first!(User::by_email(new_user.email.clone()), User, db_clone) {
        return Ok(HttpResponse::BadRequest().body("Email already in use."));
    }

    let hash = hash(&new_user.password, DEFAULT_COST).expect("Could not hash password.");

    let new_user = User::create(new_user_clone.email, hash, db.clone())
        .await
        .expect("Could not create user.");

    new_user
        .send_email_verification(
            db.clone(),
            req.connection_info().host().to_string(),
            settings.mailer_auth_token.clone(),
        )
        .await
        .expect("Could not send email verification");

    Ok(HttpResponse::Ok().finish())
}
