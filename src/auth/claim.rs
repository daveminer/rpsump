use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claim {
    pub sub: String,
    pub iat: u64,
    pub exp: u64,
}

// TODO: env var for secret
pub fn create_token(user_id: i32) -> Result<String, jsonwebtoken::errors::Error> {
    let header = Header::default();
    let payload = Claim {
        sub: user_id.to_string(),
        iat: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as u64,
        exp: (SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 3600) as u64, // expire after 1 hour
    };
    let key = EncodingKey::from_secret("secret".as_ref());
    encode(&header, &payload, &key)
}
