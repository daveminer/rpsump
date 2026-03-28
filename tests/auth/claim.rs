use chrono::Utc;
use jsonwebtoken::{decode, DecodingKey, Validation};

use rpsump::auth::claim::{create_token, Claim};

#[test]
fn test_create_token() {
    let user_id = 1;
    let private_key = "secret_key".to_string();

    let result = create_token(user_id, private_key.clone(), 15);
    assert!(result.is_ok());

    let token = result.unwrap();
    assert!(!token.is_empty());
    let decoded_token = decode::<Claim>(
        &token,
        &DecodingKey::from_secret(private_key.as_bytes()),
        &Validation::default(),
    )
    .unwrap();
    assert!(decoded_token.claims.exp <= (Utc::now().timestamp() as u64 + 15 * 60));
}
