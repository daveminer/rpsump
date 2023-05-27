mod email_verification;
mod login;
mod signup;

use actix_web::web::Data;
use serde_json::{Map, Value};

use rpsump::auth::password::Password;
use rpsump::database::DbPool;
use rpsump::models::user::User;

const TEST_EMAIL: &str = "test_acct@test.local";
const TEST_PASSWORD: &str = "testing87_*Password";

async fn create_test_user(db_pool: Data<DbPool>) -> User {
    User::create(
        TEST_EMAIL.into(),
        Password::new(TEST_PASSWORD.into()).hash().unwrap(),
        "127.0.0.1".into(),
        db_pool,
    )
    .await
    .unwrap()
}

fn signup_params() -> Map<String, Value> {
    let mut map = user_params();
    map.insert("confirm_password".into(), TEST_PASSWORD.into());
    map
}

fn user_params() -> Map<String, Value> {
    let mut map = serde_json::Map::new();
    map.insert("email".into(), TEST_EMAIL.into());
    map.insert("password".into(), TEST_PASSWORD.into());

    map
}
