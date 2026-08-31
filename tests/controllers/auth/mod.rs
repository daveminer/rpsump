use serde_json::{Map, Value};

use rpsump::repository::models::user::UserFilter;

use crate::common::test_app::TestApp;

use crate::controllers::new_user_params;

mod email_verification;
mod invite;
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
    // Enough to deserialize. Cases that expect to reach the invite lookup should
    // use `signup_params_with_invite` instead.
    map.insert("invite_token".into(), "not-a-real-invite-token".into());
    map
}

/// Issues a real invite for `email` from the seeded test user and returns
/// signup params carrying its token.
async fn signup_params_with_invite(app: &TestApp, email: &str) -> Map<String, Value> {
    let inviter = app
        .repo
        .users(UserFilter {
            email: Some(TEST_EMAIL.into()),
            ..Default::default()
        })
        .await
        .unwrap()
        .pop()
        .unwrap();

    let invite = app
        .repo
        .create_invite(email.to_string(), inviter.id)
        .await
        .unwrap();

    let mut map = serde_json::Map::new();
    map.insert("email".into(), email.into());
    map.insert("password".into(), TEST_PASSWORD.into());
    map.insert("confirm_password".into(), TEST_PASSWORD.into());
    map.insert("invite_token".into(), invite.token.into());
    map
}
