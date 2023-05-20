use actix_web::{post, web, web::Data, HttpRequest, HttpResponse, Responder, Result};

use diesel::prelude::*;
use secrecy::ExposeSecret;

use crate::auth::{hash_user_password, validate_password};
use crate::config::Settings;
use crate::controllers::auth::SignupParams;
use crate::database::{first, DbPool};
use crate::models::user::User;

#[post("/signup")]
pub async fn signup(
    req: HttpRequest,
    user_data: web::Json<SignupParams>,
    db: Data<DbPool>,
    settings: Data<Settings>,
) -> Result<impl Responder> {
    let new_user = user_data.into_inner();

    if let Err(e) = validate_password(&new_user.password, &new_user.confirm_password) {
        return Ok(HttpResponse::BadRequest().body(e.to_string()));
    };

    let db_clone = db.clone();
    let email_clone = new_user.email.clone();

    if let Ok(_user) = first!(User::by_email(new_user.email.clone()), User, db_clone) {
        return Ok(HttpResponse::BadRequest().body("Email already in use."));
    };

    let hash = match hash_user_password(&new_user.password.expose_secret()) {
        Ok(password_hash) => password_hash,
        Err(_) => {
            return Ok(HttpResponse::InternalServerError()
                .body("There was a problem; try a different password."))
        }
    };

    let ip_addr = super::ip_address(&req);
    let new_user = User::create(email_clone, hash, ip_addr, db.clone())
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
