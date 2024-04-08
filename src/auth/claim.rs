use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claim {
    pub sub: String,
    pub iat: u64,
    pub exp: u64,
}

#[tracing::instrument(skip(private_key))]
pub fn create_token(
    user_id: i32,
    private_key: String,
    duration_days: u8,
) -> Result<String, anyhow::Error> {
    let from_epoch = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
    let seconds = duration_days as u64 * 24 * 60 * 60;
    let exp_time = from_epoch.as_secs() + seconds;

    Ok(encode(
        &Header::default(),
        &Claim {
            sub: user_id.to_string(),
            exp: exp_time,
            iat: Utc::now().timestamp_millis() as u64,
        },
        &EncodingKey::from_secret(private_key.as_bytes()),
    )?)
}
