use actix_web::{dev, Error, FromRequest, HttpRequest};
use futures::future::{err, ok, Ready};
use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::auth::claim::Claim;

#[derive(Debug)]
pub struct AuthenticatedUser(pub i32);

impl FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut dev::Payload) -> Self::Future {
        let auth_header = req.headers().get("Authorization");

        if let Some(auth_header_value) = auth_header {
            let token = auth_header_value.to_str().unwrap().replace("Bearer ", "");
            let decoding_key = DecodingKey::from_secret("secret".as_ref());

            match decode::<Claim>(&token, &decoding_key, &Validation::default()) {
                Ok(token_data) => ok(AuthenticatedUser(token_data.claims.sub.parse().unwrap())),
                Err(_) => Ready::from(err(actix_web::error::ErrorUnauthorized("Invalid token"))),
            }
        } else {
            Ready::from(err(actix_web::error::ErrorUnauthorized(
                "Missing authentication",
            )))
        }
    }
}
