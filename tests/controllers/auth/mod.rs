use serde_json::{Map, Value};

use crate::controllers::new_user_params;

mod email_verification;
mod login;
mod reset_password;
mod signup;

pub const NEW_EMAIL: &str = "new_acct@test.local";
pub const TEST_EMAIL: &str = "test_acct@test.local";
pub const TEST_PASSWORD: &str = "testing87_*Password";

fn password_reset_params(token: String, new_password: String) -> Map<String, Value> {
    let mut map = serde_json::Map::new();
    map.insert("token".into(), token.into());
    map.insert("new_password".into(), new_password.clone().into());
    map.insert("new_password_confirmation".into(), new_password.into());
    map
}

fn signup_params() -> Map<String, Value> {
    let mut map = new_user_params();
    map.insert("confirm_password".into(), TEST_PASSWORD.into());
    map
}
