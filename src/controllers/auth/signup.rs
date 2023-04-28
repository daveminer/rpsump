use crate::schema::user::dsl::*;
use actix_web::{post, web, web::Data, HttpResponse, Responder, Result};
use bcrypt::{hash, DEFAULT_COST};
use diesel::RunQueryDsl;

use crate::controllers::auth::SignupParams;
use crate::database::DbPool;
use crate::email::send_email_verification;
use crate::models::user::{NewUser, User};
use crate::Settings;

#[post("/signup")]
pub async fn signup(
    user_data: web::Json<SignupParams>,
    db: Data<DbPool>,
    settings: Data<Settings>,
) -> Result<impl Responder> {
    let new_user = user_data.into_inner();

    if new_user.password.len() < 8 {
        return Ok(HttpResponse::BadRequest().body("Password must be at least 8 characters."));
    }

    if new_user.password != new_user.confirm_password {
        return Ok(HttpResponse::BadRequest().body("Password and confirmation do not match."));
    }

    let hash = hash(&new_user.password, DEFAULT_COST).expect("Could not hash password.");

    let mut conn = db.get().expect("Could not get db connection.");

    let users: Vec<User> = diesel::insert_into(user)
        .values(&NewUser {
            email: new_user.email,
            password_hash: hash,
        })
        .get_results(&mut conn)
        .expect("Could not insert new user");

    send_email_verification(users[0].clone(), db, settings.mailer_auth_token.clone())
        .await
        .expect("Could not send email verification");

    Ok(HttpResponse::Ok().finish())
}
