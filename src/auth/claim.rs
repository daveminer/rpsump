use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

pub const TOKEN_EXPIRATION_TIME_SECONDS: u64 = 60 * 60 * 24;
#[derive(Debug, Serialize, Deserialize)]
pub struct Claim {
    pub sub: String,
    pub iat: u64,
    pub exp: u64,
}

pub fn create_token(
    user_id: i32,
    private_key: String,
) -> Result<String, jsonwebtoken::errors::Error> {
    let exp_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Could not get current time")
        .as_secs()
        + TOKEN_EXPIRATION_TIME_SECONDS;

    encode(
        &Header::default(),
        &Claim {
            sub: user_id.to_string(),
            exp: exp_time,
            iat: Utc::now().timestamp_millis() as u64,
        },
        &EncodingKey::from_secret(private_key.as_bytes()),
    )
}
