use actix_web::web::Data;
use anyhow::Error;
use rpsump::auth::hash_user_password;
use rpsump::models::user_event::UserEvent;
use serde_json::Value;

use rpsump::controllers::ErrorBody;
use rpsump::database::DbPool;
use rpsump::models::user::User;
use rpsump::models::user_event::EventType;

use crate::common::test_app::spawn_app;

const TEST_EMAIL: &str = "test_acct@test.local";
const TEST_PASSWORD: &str = "testing87_*Password";

#[tokio::test]
async fn login_failed_password_too_short() {
    // Arrange
    let app = spawn_app().await;
    let mut params = user_params();
    params["password"] = "test".into();

    // Act
    let response = app.post_login(&params).await;
    let status = response.status();
    let body: ErrorBody = response.json().await.unwrap();

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
    let body: ErrorBody = response.json().await.unwrap();

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
    let body: ErrorBody = response.json().await.unwrap();

    // Assert
    assert!(status.is_client_error());
    assert_eq!(body.reason, "Invalid email or password.");
}

#[tokio::test]
async fn login_missing_email() {
    // Arrange
    let app = spawn_app().await;
    let db_pool = app.db_pool.clone();
    let _user = create_test_user(Data::new(db_pool.clone())).await;
    let mut params = user_params();
    params["email"] = "".into();

    // Act
    let response = app.post_login(&params).await;
    let status = response.status();
    let body: ErrorBody = response.json().await.unwrap();

    // Assert
    assert!(status.is_client_error());
    assert_eq!(body.reason, "Email and password are required.");
}

#[tokio::test]
async fn login_missing_password() {
    // Arrange
    let app = spawn_app().await;
    let db_pool = app.db_pool.clone();
    let _user = create_test_user(Data::new(db_pool.clone())).await;
    let mut params = user_params();
    params["password"] = "".into();

    // Act
    let response = app.post_login(&params).await;
    let status = response.status();
    let body: ErrorBody = response.json().await.unwrap();

    // Assert
    assert!(status.is_client_error());
    assert_eq!(body.reason, "Email and password are required.");
}

#[tokio::test]
async fn login_success() {
    // Arrange
    let app = spawn_app().await;
    let db_pool = app.db_pool.clone();
    let user = create_test_user(Data::new(db_pool.clone())).await;

    // Act
    let response = app.post_login(&user_params()).await;
    let status = response.status();
    let body: Value = response.json().await.unwrap();

    // Assert
    assert!(status.is_success());
    assert!(body["token"].is_string());
    let events = recent_login_events(user, db_pool).await.unwrap();
    assert_eq!(events.len(), 1);
}

#[tokio::test]
async fn login_user_blocked() {
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
        hash_user_password(TEST_PASSWORD.into()).unwrap(),
        "127.0.0.1".into(),
        db_pool,
    )
    .await
    .unwrap()
}

async fn recent_login_events(record: User, db_pool: DbPool) -> Result<Vec<UserEvent>, Error> {
    UserEvent::recent_events(
        Some(record),
        None,
        EventType::Login,
        10,
        actix_web::web::Data::new(db_pool),
    )
    .await
}

fn user_params() -> Value {
    serde_json::json!({
        "email": TEST_EMAIL,
        "password": TEST_PASSWORD,
    })
}
