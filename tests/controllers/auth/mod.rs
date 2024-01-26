use serde_json::{Map, Value};

use super::{TEST_EMAIL, TEST_PASSWORD};
use crate::controllers::user_params;
use rpsump::auth::password::Password;
use rpsump::repository::{models::user::User, Repo};

mod email_verification;
mod login;
mod reset_password;
mod signup;

// TODO: Move this to a common module
pub async fn create_test_user(repo: Repo) -> User {
    repo.create_user(
        TEST_EMAIL.into(),
        Password::new(TEST_PASSWORD.into()).hash().unwrap(),
        "127.0.0.1".into(),
    )
    .await
    .expect("Could not create test user")
}

fn password_reset_params(token: String, new_password: String) -> Map<String, Value> {
    let mut map = serde_json::Map::new();
    map.insert("token".into(), token.into());
    map.insert("new_password".into(), new_password.clone().into());
    map.insert("new_password_confirmation".into(), new_password.into());
    map
}

fn signup_params() -> Map<String, Value> {
    let mut map = user_params();
    map.insert("confirm_password".into(), TEST_PASSWORD.into());
    map
}
