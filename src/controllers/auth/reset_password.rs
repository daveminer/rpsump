use actix_web::{post, web, web::Data, HttpRequest, HttpResponse, Responder, Result};
use diesel::RunQueryDsl;

use crate::database::{first, DbPool};
use crate::email;
use crate::models::user::User;
use crate::Settings;

#[post("/reset_password")]
async fn reset_password(
    req: HttpRequest,
    email: web::Json<String>,
    db: Data<DbPool>,
    settings: Data<Settings>,
) -> Result<impl Responder> {
    let db_clone = db.clone();

    let user: User = match first!(User::by_email(email.to_string()), User, db.clone()) {
        Ok(user) => user,
        Err(_) => return Ok(HttpResponse::Ok().finish()),
    };

    email::send_password_reset(
        user,
        db_clone,
        req.connection_info().host().to_string(),
        settings.mailer_auth_token.clone(),
    )
    .await
    .expect("Could not send password reset email");

    Ok(HttpResponse::Ok().finish())
}

// TODO GET password
