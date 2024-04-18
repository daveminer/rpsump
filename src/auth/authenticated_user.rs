use actix_web::{dev, error, web, Error, FromRequest, HttpRequest};
use anyhow::anyhow;
use futures::future::err;
use jsonwebtoken::{decode, DecodingKey, TokenData, Validation};
use std::future::Future;
use std::pin::Pin;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::auth::claim::Claim;
use crate::config::Settings;

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
        let token = match auth_token_from_request(req) {
            Ok(token) => token,
            Err(e) => {
                return unauthorized_err(&e.to_string());
            }
        };

        if settings(req).is_none() {
            tracing::error!(
                target = module_path!(),
                "Configuration error; settings are None."
            );
            return internal_server_error();
        }

        let user = match parse_token(token, settings(req).unwrap()) {
            Ok(user) => user,
            Err(e) => {
                return unauthorized_err(&e.to_string());
            }
        };

        tracing::info!(
            target = module_path!(),
            user_id = user.id,
            "Authenticated user"
        );
        Box::pin(async move { Ok(AuthenticatedUser { id: user.id }) })
    }
}

#[tracing::instrument(skip(req))]
fn auth_token_from_request(req: &HttpRequest) -> Result<String, anyhow::Error> {
    let auth_header = req.headers().get("Authorization");
    if auth_header.is_none() {
        return Err(anyhow!("Missing authentication."));
    };

    let token = match auth_header.unwrap().to_str() {
        Ok(token) => token.replace("Bearer ", ""),
        Err(e) => {
            tracing::error!(
                target = module_path!(),
                error = e.to_string(),
                "Could not parse token"
            );
            return Err(anyhow!("Invalid authentication."));
        }
    };

    Ok(token)
}

#[tracing::instrument(skip(token, settings))]
fn parse_token(token: String, settings: &Settings) -> Result<AuthenticatedUser, Error> {
    match decode::<Claim>(&token, &decoding_key(settings), &Validation::default()) {
        Ok(token) => {
            match token_expired(&token) {
                Ok(true) => return Err(error::ErrorUnauthorized("Token expired")),
                Ok(false) => (),
                Err(e) => {
                    tracing::error!(
                        target = module_path!(),
                        error = e.to_string(),
                        "Error while checking token expiry"
                    );
                    return Err(error::ErrorInternalServerError("Internal server error."));
                }
            }

            Ok(AuthenticatedUser {
                id: token.claims.sub.parse().unwrap(),
            })
        }
        Err(_e) => Err(error::ErrorUnauthorized("Invalid token")),
    }
}

#[tracing::instrument(skip(settings))]
fn decoding_key(settings: &Settings) -> DecodingKey {
    DecodingKey::from_secret(settings.jwt_secret.as_ref())
}

#[tracing::instrument(skip(req))]
fn settings(req: &HttpRequest) -> Option<&Settings> {
    match req.app_data::<web::Data<Settings>>() {
        Some(settings) => Some(settings.get_ref()),
        None => None,
    }
}

#[tracing::instrument(skip(token_expiry))]
fn token_expired(token_expiry: &TokenData<Claim>) -> Result<bool, anyhow::Error> {
    let now = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(dur) => dur.as_secs(),
        Err(e) => {
            tracing::error!(
                target = module_path!(),
                error = e.to_string(),
                "Error while finding duration."
            );
            return Err(anyhow!("Could not get current time."));
        }
    };

    Ok(token_expiry.claims.exp < now)
}

fn internal_server_error() -> AuthFuture {
    Box::pin(err(actix_web::error::ErrorInternalServerError(
        "Internal server error.",
    )))
}

fn unauthorized_err(message: &str) -> AuthFuture {
    Box::pin(err(actix_web::error::ErrorUnauthorized(
        message.to_string(),
    )))
}
