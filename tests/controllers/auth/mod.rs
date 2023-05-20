mod login;
mod logout;

use actix_web::web::Data;
use serde_json::Value;

use rpsump::auth::hash_user_password;
use rpsump::database::DbPool;
use rpsump::models::user::User;

const TEST_EMAIL: &str = "test_acct@test.local";
const TEST_PASSWORD: &str = "testing87_*Password";

async fn create_test_user(db_pool: Data<DbPool>) -> User {
    User::create(
        TEST_EMAIL.into(),
        hash_user_password(TEST_PASSWORD.into()).unwrap(),
        "127.0.0.1".into(),
        db_pool,
    )
    .await
    .unwrap()
}

async fn create_logged_in_user(db_pool: Data<DbPool>) {
    let user = create_test_user(db_pool).await;
}

fn user_params() -> Value {
    serde_json::json!({
        "email": TEST_EMAIL,
        "password": TEST_PASSWORD,
    })
}
