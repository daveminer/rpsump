use actix_web::{post, web, web::Data, HttpRequest, HttpResponse, Responder, Result};
use serde::Deserialize;
use validator::Validate;

use crate::auth::{password::Password, validate_password};
use crate::config::Settings;
use crate::controllers::auth::ip_address;
use crate::database::DbPool;
use crate::models::user::User;

#[derive(Debug, Deserialize, Validate)]
pub struct SignupParams {
    #[validate(email)]
    email: String,
    #[validate(custom = "validate_password")]
    password: Password,
    #[validate(must_match = "password")]
    confirm_password: Password,
}

#[post("/signup")]
pub async fn signup(
    req: HttpRequest,
    params: web::Json<SignupParams>,
    db: Data<DbPool>,
    settings: Data<Settings>,
) -> Result<impl Responder> {
    // Validate params
    match &params.validate() {
        Ok(_) => (),
        Err(e) => return Ok(HttpResponse::BadRequest().body(e.to_string())),
    };

    // Hash password
    let hash = match params.password.hash() {
        Ok(password_hash) => password_hash,
        Err(_) => return Ok(HttpResponse::BadRequest().body("Try a different password.")),
    };

    // Create user
    let new_user =
        match User::create(params.email.to_string(), hash, ip_address(&req), db.clone()).await {
            Ok(user) => user,
            Err(e) => {
                println!("EEE: {}", e);
                return Ok(HttpResponse::BadRequest()
                    .body("There was a problem; try a different email address."));
            }
        };

    // Send email verification
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
