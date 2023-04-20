use actix_web::{dev, Error, FromRequest, HttpRequest};
use futures::future::{err, ok, Ready};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct AuthenticatedUser(pub i32);

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    iat: usize,
    exp: usize,
}

pub fn create_token(user_id: i32) -> Result<String, jsonwebtoken::errors::Error> {
    let header = Header::default();
    let payload = Claims {
        sub: user_id.to_string(),
        iat: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize,
        exp: (SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 3600) as usize, // expire after 1 hour
    };
    let key = EncodingKey::from_secret("secret".as_ref());
    encode(&header, &payload, &key)
}

impl FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut dev::Payload) -> Self::Future {
        let auth_header = req.headers().get("Authorization");

        if let Some(auth_header_value) = auth_header {
            let token = auth_header_value.to_str().unwrap().replace("Bearer ", "");
            let decoding_key = DecodingKey::from_secret("secret".as_ref());

            match decode::<Claims>(&token, &decoding_key, &Validation::default()) {
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
