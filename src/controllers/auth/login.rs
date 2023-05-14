use actix_identity::Identity;
use actix_web::error;
use actix_web::{post, web, web::Data, HttpMessage, HttpRequest, HttpResponse, Responder, Result};
use bcrypt::verify;
use diesel::prelude::*;
use diesel::result::Error;
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};

use crate::auth::claim::create_token;
use crate::auth::validate_password_length;
use crate::config::Settings;
use crate::controllers::auth::AuthParams;
use crate::controllers::ErrorResponse;
use crate::database::{first, DbPool};
use crate::models::user_event::*;
use crate::schema::user_event;

use crate::models::user::User;

const BAD_CREDS: &str = "Invalid email or password.";

#[derive(Serialize, Deserialize)]
struct Response {
    token: String,
}

#[post("/login")]
pub async fn login(
    request: HttpRequest,
    user_data: web::Json<AuthParams>,
    db: Data<DbPool>,
    settings: Data<Settings>,
) -> Result<impl Responder> {
    // User lookup from params
    let AuthParams { email, password } = user_data.into_inner();
    let db_clone = db.clone();
    let (user, existing_user_password_hash) = match first!(User::by_email(email), User, db_clone) {
        Ok(user) => (Some(user.clone()), user.password_hash),
        Err(_e) => (None, "".to_string()),
    };

    // TODO: DRY this up with signup
    let conn_info = request.connection_info();

    let ip_addr = conn_info.peer_addr().expect("Could not get IP address.");

    if let Err(e) = UserEvent::check_allowed_status(
        user.clone(),
        ip_addr.to_string(),
        settings.auth_attempts_allowed,
        db.clone(),
    )
    .await
    {
        return Err(error::ErrorUnauthorized(e));
    }

    if let Err(e) = validate_password_length(&password.expose_secret()) {
        return Ok(HttpResponse::BadRequest().json(ErrorResponse {
            reason: e.to_string(),
        }));
    }

    // Resist timing attacks by always hashing the password
    if let Err(_e) = verify(password.expose_secret(), &existing_user_password_hash) {
        return Ok(HttpResponse::BadRequest().json(ErrorResponse {
            reason: BAD_CREDS.into(),
        }));
    }

    let user_id = user.unwrap().id;
    let conn_info = request.connection_info();
    let ip_addr = conn_info.peer_addr().expect("Could not get IP address.");

    Identity::login(&request.extensions(), user_id.to_string()).expect("Could not log identity in");

    let mut conn = db.get().expect("Could not get a db connection.");

    let _user_event = conn.transaction::<_, Error, _>(|conn| {
        let user_event: UserEvent = diesel::insert_into(user_event::table)
            .values((
                user_event::user_id.eq(user_id),
                user_event::event_type.eq(EventType::Signup.to_string()),
                user_event::ip_address.eq(ip_addr.clone()),
            ))
            .get_result(conn)?;

        Identity::login(&request.extensions(), user_id.to_string())
            .expect("Could not log identity in");

        Ok(user_event)
    });

    let token = create_token(user_id, settings.jwt_secret.clone())
        .expect("Could not create token for user.");

    Ok(HttpResponse::Ok().json(Response { token }))
}
