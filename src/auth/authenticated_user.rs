use crate::auth::claim::Claim;
use crate::config::Settings;

use actix_web::{dev, error, http::header::HeaderValue, web, Error, FromRequest, HttpRequest};
use futures::future::err;
use jsonwebtoken::{decode, DecodingKey, TokenData, Validation};
use std::future::Future;
use std::pin::Pin;
use std::time::{SystemTime, UNIX_EPOCH};

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
            Err(e) => {
                tracing::warn!("Could not get user from token: {:?}", e);
                return unauthorized_err("Invalid token".to_string());
            }
        };

        Box::pin(async move { Ok(AuthenticatedUser { id: user.id }) })
    }
}

fn user_from_token(
    auth_header: Option<&HeaderValue>,
    settings: &Settings,
) -> Result<AuthenticatedUser, Error> {
    if auth_header.is_none() {
        tracing::warn!("Authentication token not found for request.");
        return Err(error::ErrorUnauthorized("Missing authentication"));
    };

    let encoded_token = auth_header
        .unwrap()
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

fn unauthorized_err(message: String) -> AuthFuture {
    Box::pin(err(actix_web::error::ErrorUnauthorized(message)))
}
