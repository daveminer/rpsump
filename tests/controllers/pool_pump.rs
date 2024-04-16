use serde_json::{json, Value};

use crate::common::test_app::spawn_app;
use crate::controllers::auth::create_test_user;
use crate::controllers::user_params;

#[tokio::test]
async fn pool_pump_success() {
    // Arrange
    let app = spawn_app().await;
    let _user = create_test_user(app.repo).await;

    // Act
    let response = app.post_login(&user_params()).await;
    let body: Value = response.json().await.unwrap();
    let token = body["token"].as_str().unwrap();

    let low_response = app
        .post_pool_pump(token.to_string(), json!({"speed": "low"}))
        .await;
    let low_response_body: Value = low_response.json().await.unwrap();

    let max_response = app
        .post_pool_pump(token.to_string(), json!({"speed": "max"}))
        .await;
    let max_response_body: Value = max_response.json().await.unwrap();

    let off_response = app
        .post_pool_pump(token.to_string(), json!({"speed": "off"}))
        .await;
    let off_response_body: Value = off_response.json().await.unwrap();

    // Assert
    assert_eq!(low_response_body["status"].as_str(), Some("ok"));
    assert_eq!(max_response_body["status"].as_str(), Some("ok"));
    assert_eq!(off_response_body["status"].as_str(), Some("ok"));
}

#[tokio::test]
async fn pool_pump_failed_no_auth() {
    // Arrange
    let app = spawn_app().await;

    let sump_event_response = app
        .post_pool_pump("123".to_string(), json!({"speed": "low"}))
        .await;

    // Assert
    assert!(sump_event_response.status() == 401);
}
