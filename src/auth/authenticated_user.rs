use actix_web::{dev, error, http::header::HeaderValue, web, Error, FromRequest, HttpRequest};
use diesel::RunQueryDsl;
// Replacing this might allow for removal of the futures crate.
use futures::future::err;
use jsonwebtoken::{decode, DecodingKey, TokenData, Validation};
use std::future::Future;
use std::pin::Pin;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::auth::claim::Claim;
use crate::config::Settings;
use crate::database::DbPool;
use crate::first;
use crate::models::user::User;

#[derive(Debug)]
pub struct AuthenticatedUser {
    pub id: i32,
}

type AuthFuture = <AuthenticatedUser as FromRequest>::Future;

impl FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    #[tracing::instrument(skip(req, _payload))]
    fn from_request(req: &HttpRequest, _payload: &mut dev::Payload) -> Self::Future {
        let auth_header = req.headers().get("Authorization");
        let user = match user_from_token(auth_header, settings(req)) {
            Ok(user) => user,
            Err(_e) => return unauthorized_err("Invalid token".to_string()),
        };

        let database = req.app_data::<web::Data<DbPool>>().unwrap().get_ref();
        let settings = req.app_data::<web::Data<Settings>>().unwrap().get_ref();
        validate_user(user, database, settings)
    }
}

fn user_from_token(
    auth_header: Option<&HeaderValue>,
    settings: &Settings,
) -> Result<AuthenticatedUser, Error> {
    if auth_header.is_none() {
        return Err(error::ErrorUnauthorized("Missing authentication"));
    };

    let encoded_token = auth_header
        .expect("Could not convert token to string")
        .to_str()
        .unwrap()
        .replace("Bearer ", "");

    parse_token(encoded_token, settings)
}

fn parse_token(token: String, settings: &Settings) -> Result<AuthenticatedUser, Error> {
    match decode::<Claim>(&token, &decoding_key(settings), &Validation::default()) {
        Ok(token) => {
            if token_expired(&token) {
                return Err(error::ErrorUnauthorized("Token expired"));
            }

            Ok(AuthenticatedUser {
                id: token.claims.sub.parse().unwrap(),
            })
        }
        Err(_) => Err(error::ErrorUnauthorized("Invalid token")),
    }
}

fn decoding_key(settings: &Settings) -> DecodingKey {
    DecodingKey::from_secret(settings.jwt_secret.as_ref())
}

fn settings(req: &HttpRequest) -> &Settings {
    req.app_data::<web::Data<Settings>>()
        .expect("Could not get settings")
        .get_ref()
}

fn token_expired(token_expiry: &TokenData<Claim>) -> bool {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Could not get current time")
        .as_secs();

    token_expiry.claims.exp < now
}

fn validate_user(user: AuthenticatedUser, db: &DbPool, settings: &Settings) -> AuthFuture {
    let db_clone = db.clone();
    let settings_clone = settings.clone();
    Box::pin(async move {
        match first!(User::by_id(user.id), User, db_clone) {
            Ok(user) => validate_activated_status(user, &settings_clone),
            Err(_) => Err(error::ErrorUnauthorized("Invalid token")),
        }
    })
}

fn validate_activated_status(
    user: User,
    settings: &Settings,
) -> Result<AuthenticatedUser, actix_web::Error> {
    if user.activated || !settings.user_activation_required {
        Ok(AuthenticatedUser { id: user.id })
    } else {
        Err(error::ErrorUnauthorized("User is not active"))
    }
}

fn unauthorized_err(message: String) -> AuthFuture {
    Box::pin(err(actix_web::error::ErrorUnauthorized(message)))
}
