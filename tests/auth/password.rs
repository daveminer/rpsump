use secrecy::ExposeSecret;
use serde_json;

use rpsump::auth::password::Password;

#[test]
fn test_password_hash() {
    let password = Password::new("password123".to_string());
    let hash_result = password.hash();
    assert!(hash_result.is_ok());
    let hash = hash_result.unwrap();
    assert_ne!(hash, "password123"); // Hashed value should be different from the original password
}

#[test]
fn test_password_new() {
    let password = Password::new("password123".to_string());
    assert_eq!(password.expose_secret().as_str(), "password123");
}

#[test]
fn test_password_debug() {
    let password = Password::new("password123".to_string());
    assert_eq!(format!("{:?}", password), "[REDACTED]");
}

#[test]
fn test_password_eq() {
    let password1 = Password::new("password123".to_string());
    let password2 = Password::new("password123".to_string());
    assert_eq!(password1, password2);
}

#[test]
fn test_password_serialize() {
    let password = Password::new("password123".to_string());
    let serialized = serde_json::to_string(&password).unwrap();
    assert_eq!(serialized, "\"password123\"");
}
