use anyhow::Error;
use rpsump::{repository::models::user_event::UserEvent, repository::Repo};
use serde_json::Value;

use rpsump::repository::models::{user::User, user_event::EventType};
use rpsump::util::ApiResponse;

use super::{create_test_user, user_params};
use crate::common::test_app::spawn_app;

#[tokio::test]
async fn login_failed_username_not_found() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app.post_login(&user_params()).await;
    let status = response.status();
    let body: ApiResponse = response.json().await.unwrap();

    // Assert
    assert!(status.is_client_error());
    assert_eq!(body.message, "Invalid email or password.");
}

#[tokio::test]
async fn login_password_incorrect() {
    // Arrange
    let app = spawn_app().await;
    let _user = create_test_user(app.repo).await;
    let mut params = user_params();
    params["password"] = "wrong_password".into();

    // Act
    let response = app.post_login(&params).await;
    let status = response.status();
    let body: ApiResponse = response.json().await.unwrap();

    // Assert
    assert!(status.is_client_error());
    assert_eq!(body.message, "Invalid email or password.");
}

#[tokio::test]
async fn login_missing_email() {
    // Arrange
    let app = spawn_app().await;
    let _user = create_test_user(app.repo).await;
    let mut params = user_params();
    params["email"] = "".into();

    // Act
    let response = app.post_login(&params).await;
    let status = response.status();
    let body: ApiResponse = response.json().await.unwrap();

    // Assert
    assert!(status.is_client_error());
    assert_eq!(body.message, "Email and password are required.");
}

#[tokio::test]
async fn login_missing_password() {
    // Arrange
    let app = spawn_app().await;
    let _user = create_test_user(app.repo).await;
    let mut params = user_params();
    params["password"] = "".into();

    // Act
    let response = app.post_login(&params).await;
    let status = response.status();
    let body: ApiResponse = response.json().await.unwrap();

    // Assert
    assert!(status.is_client_error());
    assert_eq!(body.message, "Email and password are required.");
}

#[tokio::test]
async fn login_success() {
    // Arrange
    let app = spawn_app().await;
    let user = create_test_user(app.repo).await;

    // Act
    let response = app.post_login(&user_params()).await;
    let status = response.status();
    let body: Value = response.json().await.unwrap();

    // Assert
    assert!(status.is_success());
    assert!(body["token"].is_string());
    let events = recent_login_events(user.clone(), app.repo).await.unwrap();
    assert_eq!(events.len(), 1);
}

async fn recent_login_events(record: User, repo: Repo) -> Result<Vec<UserEvent>, Error> {
    repo.user_events(record.id, Some(EventType::Login), 10)
        .await
}
