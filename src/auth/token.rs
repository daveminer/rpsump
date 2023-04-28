use chrono::{DateTime, Duration, Utc};
use rand::{distributions::Alphanumeric, Rng};

const EMAIL_CONFIRM_TOKEN_VALIDITY_DURATION: i64 = 3600 * 24;
const PASSWORD_RESET_TOKEN_VALIDITY_DURATION: i64 = 3600; // 1 hour in seconds
const TOKEN_LENGTH: usize = 32;

#[derive(Clone)]
pub struct Token {
    pub value: String,
    pub user_id: i32,
    pub expires_at: DateTime<Utc>,
}

impl Token {
    pub fn new_email_verification(user_id: i32) -> Self {
        Self::new_token(user_id, EMAIL_CONFIRM_TOKEN_VALIDITY_DURATION)
    }

    pub fn new_password_reset(user_id: i32) -> Self {
        Self::new_token(user_id, PASSWORD_RESET_TOKEN_VALIDITY_DURATION)
    }

    fn new_token(user_id: i32, duration: i64) -> Self {
        let now = Utc::now();
        let expires_at = now + Duration::seconds(duration);
        let value: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(TOKEN_LENGTH)
            .map(char::from)
            .collect();

        Self {
            value,
            user_id,
            expires_at,
        }
    }
}
