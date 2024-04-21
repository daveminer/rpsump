use anyhow::Error;

use rpsump::test_fixtures::gpio::build_mock_gpio;
use rpsump::{repository::models::user_event::UserEvent, repository::Repo};

use rpsump::repository::models::{user::User, user_event::EventType};
use rpsump::util::ApiResponse;

use crate::common::test_app::spawn_app;
use crate::controllers::user_params;

#[tokio::test]
async fn login_failed_username_not_found() {
    // Arrange
    let app = spawn_app(&build_mock_gpio()).await;

    let mut map = serde_json::Map::new();
    map.insert("email".into(), "not-found-email@test.com".into());
    map.insert("password".into(), "test-password".into());

    // Act
    let response = app.post_login(&map).await;
    let status = response.status();
    let body: ApiResponse = response.json().await.unwrap();

    // Assert
    assert!(status.is_client_error());
    assert_eq!(body.message, "Invalid email or password.");
}

#[tokio::test]
async fn login_password_incorrect() {
    // Arrange
    let app = spawn_app(&build_mock_gpio()).await;
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
    let app = spawn_app(&build_mock_gpio()).await;
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
    let app = spawn_app(&build_mock_gpio()).await;
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
    let app = spawn_app(&build_mock_gpio()).await;

    // Act
    let response = app.post_login(&user_params()).await;
    println!("RESPONSE {:?}", response.text().await.unwrap());
    //let status = response.status();
    //let body: Value = response.json().await.unwrap();

    // // Assert
    // assert!(status.is_success());
    // assert!(body["token"].is_string());

    // let user_filter = UserFilter {
    //     email: Some(TEST_EMAIL.into()),
    //     ..Default::default()
    // };
    // let user = &app.repo.users(user_filter).await.unwrap()[0];
    // let events = recent_login_events(user.clone(), app.repo).await.unwrap();
    // assert_eq!(events.len(), 1);
}

async fn recent_login_events(record: User, repo: Repo) -> Result<Vec<UserEvent>, Error> {
    repo.user_events(record.id, Some(EventType::Login), 10)
        .await
}
