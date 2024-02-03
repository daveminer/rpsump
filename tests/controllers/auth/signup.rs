use anyhow::Error;

use rpsump::repository::{
    models::{
        user::UserFilter,
        user_event::{EventType, UserEvent},
    },
    Repo,
};
use rpsump::util::ApiResponse;

use super::{create_test_user, signup_params, user_params};
use crate::common::test_app::spawn_app;
use crate::controllers::mock_email_verification_send;

#[tokio::test]
async fn signup_failed_email_taken() {
    // Arrange
    let app = spawn_app().await;
    let user = create_test_user(app.repo).await;
    let mut params = signup_params();
    params["email"] = serde_json::json!(user.email);

    // Act
    let response = app.post_signup(&params).await;
    let status = response.status();
    let body: ApiResponse = response.json().await.unwrap();

    // Assert
    assert!(status.is_client_error());
    assert_eq!(body.message, "Email already exists.");
}

#[tokio::test]
async fn signup_failed_password_does_not_match() {
    // Arrange
    let app = spawn_app().await;
    let mut params = signup_params();
    params["confirm_password"] = "not-matching".into();

    // Act
    let response = app.post_signup(&params).await;
    let status = response.status();
    let body: ApiResponse = response.json().await.unwrap();

    // Assert
    assert!(status.is_client_error());
    assert_eq!(
        body.message,
        "confirm_password: Password and confirm password must match."
    );
}

#[tokio::test]
async fn signup_failed_missing_confirm_password() {
    // Arrange
    let app = spawn_app().await;
    let params = user_params();

    // Act
    let response = app.post_signup(&params).await;
    let status = response.status();

    let body: ApiResponse = response.json().await.unwrap();

    // Assert
    assert!(status.is_client_error());
    assert_eq!(
        body.message,
        "Json deserialize error: missing field `confirm_password` at line 1 column 65"
    );
}

#[tokio::test]
async fn signup_success() {
    // Arrange
    let app = spawn_app().await;
    let params = signup_params();
    let email = params.get("email").unwrap().as_str().unwrap();
    let _mock = mock_email_verification_send(&app).await;

    // Act
    let response = app.post_signup(&params).await;
    let status = response.status();
    let body: ApiResponse = response.json().await.unwrap();

    // Assert
    assert!(status.is_success());
    assert_eq!(body.message, "User created.");

    let events = recent_signup_events(email.to_string(), app.repo)
        .await
        .unwrap();
    assert_eq!(events.len(), 1);
}

async fn recent_signup_events(email: String, repo: Repo) -> Result<Vec<UserEvent>, Error> {
    //let mut conn = db_pool.get().unwrap();
    //let user = User::by_email(email).first(&mut conn).unwrap();
    let user_filter = UserFilter {
        email: Some(email),
        ..Default::default()
    };

    let user = repo.users(user_filter).await.unwrap().pop().unwrap();

    repo.user_events(user.id, Some(EventType::Signup), 10).await
}
