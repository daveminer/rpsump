use actix_web::web::Data;
use serde_json::Value;

use rpsump::controllers::ErrorResponse;
use rpsump::database::DbPool;
use rpsump::models::user::User;

use crate::common::test_app::spawn_app;

const TEST_EMAIL: &str = "test_acct@test.local";
const TEST_PASSWORD: &str = "testing_*Password";

#[tokio::test]
async fn login_failed_password_too_short() {
    // Arrange
    let app = spawn_app().await;
    let mut params = user_params();
    params["password"] = "test".into();

    // Act
    let response = app.post_login(&params).await;
    let status = response.status();
    let body: ErrorResponse = response.json().await.unwrap();

    // Assert
    assert!(status.is_client_error());
    assert_eq!(body.reason, "Password is too short.");
}

#[tokio::test]
async fn login_failed_username_not_found() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app.post_login(&user_params()).await;
    let status = response.status();
    let body: ErrorResponse = response.json().await.unwrap();

    // Assert
    assert!(status.is_client_error());
    assert_eq!(body.reason, "Invalid email or password.");
}

#[tokio::test]
async fn login_password_incorrect() {
    // Arrange
    let app = spawn_app().await;
    let db_pool = app.db_pool.clone();
    let _user = create_test_user(Data::new(db_pool.clone())).await;
    let mut params = user_params();
    params["password"] = "wrong_password".into();

    // Act
    let response = app.post_login(&params).await;
    let status = response.status();
    let body: ErrorResponse = response.json().await.unwrap();

    // Assert
    assert!(status.is_client_error());
    assert_eq!(body.reason, "Invalid email or password.");
}

#[tokio::test]
async fn login_success() {
    // Arrange
    let app = spawn_app().await;
    let db_pool = app.db_pool.clone();
    let _user = create_test_user(Data::new(db_pool.clone())).await;

    // Act
    let response = app.post_login(&user_params()).await;
    let status = response.status();
    let body: Value = response.json().await.unwrap();

    // Assert
    assert!(status.is_success());
    assert!(body["token"].is_string());
}

async fn create_test_user(db_pool: Data<DbPool>) -> User {
    User::create(
        TEST_EMAIL.into(),
        TEST_PASSWORD.into(),
        "127.0.0.1".into(),
        db_pool,
    )
    .await
    .unwrap()
}

fn user_params() -> Value {
    serde_json::json!({
        "email": TEST_EMAIL,
        "password": TEST_PASSWORD,
    })
}
