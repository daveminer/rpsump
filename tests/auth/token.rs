use chrono::{Duration, Utc};
use rpsump::auth::token::{
    Token, EMAIL_CONFIRM_TOKEN_VALIDITY_DURATION, PASSWORD_RESET_TOKEN_VALIDITY_DURATION,
    TOKEN_LENGTH,
};

#[test]
fn test_new_email_verification() {
    let token = Token::new_email_verification(1);
    assert_eq!(token.user_id, 1);
    assert_eq!(token.value.len(), TOKEN_LENGTH);
    assert!(token.value.chars().all(|c| c.is_alphanumeric()));
    assert!(
        token.expires_at - Duration::seconds(EMAIL_CONFIRM_TOKEN_VALIDITY_DURATION)
            < Utc::now().naive_utc()
    );
}

#[test]
fn test_new_password_reset() {
    let token = Token::new_password_reset(1);
    assert_eq!(token.user_id, 1);
    assert_eq!(token.value.len(), TOKEN_LENGTH);
    assert!(token.value.chars().all(|c| c.is_alphanumeric()));
    assert!(
        token.expires_at - Duration::seconds(PASSWORD_RESET_TOKEN_VALIDITY_DURATION)
            < Utc::now().naive_utc()
    );
}
