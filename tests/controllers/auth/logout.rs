use rpsump::controllers::ErrorBody;

use super::user_params;
use crate::common::test_app::spawn_app;

#[tokio::test]
async fn logout_success() {
    // Arrange
    let app = spawn_app().await;
    let mut params = user_params();
    params["password"] = "test".into();

    // Act
    let response = app.post_logout().await;
    let status = response.status();
    let body: ErrorBody = response.json().await.unwrap();

    // Assert
    assert!(status.is_success());
    assert_eq!(body.message, "Password is too short.");
}

#[tokio::test]
async fn logout_failed_user_not_logged_in() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app.post_logout().await;
    let status = response.status();
    let body: ErrorBody = response.json().await.unwrap();

    // Assert
    assert!(status.is_client_error());
    assert_eq!(body.message, "Invalid email or password.");
}

// async fn recent_login_events(record: User, db_pool: DbPool) -> Result<Vec<UserEvent>, Error> {
//     UserEvent::recent_events(
//         Some(record),
//         None,
//         EventType::Login,
//         10,
//         actix_web::web::Data::new(db_pool),
//     )
//     .await
// }

// fn user_params() -> Value {
//     serde_json::json!({
//         "email": TEST_EMAIL,
//         "password": TEST_PASSWORD,
//     })
// }
