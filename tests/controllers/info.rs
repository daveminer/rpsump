use serde_json::Value;

use crate::common::fixtures::sump_event::insert_sump_events;
use crate::common::test_app::spawn_app;
use crate::controllers::auth::create_test_user;
use crate::controllers::user_params;

#[tokio::test]
async fn info_success() {
    // Arrange
    let app = spawn_app().await;
    let _user = create_test_user(app.repo).await;
    let _sump_events = insert_sump_events(app.repo).await;

    // Act
    let response = app.post_login(&user_params()).await;
    let body: Value = response.json().await.unwrap();
    let token = body["token"].as_str().unwrap();

    let sump_event_response = app.get_info(token.to_string()).await;
    let response: Value = sump_event_response.json().await.unwrap();

    // Assert
    assert!(response["heater"].as_bool() == Some(false));
    assert!(response["poolPumpSpeed"].as_str() == Some("off"));
}

#[tokio::test]
async fn info_failed_no_auth() {
    let app = spawn_app().await;
    let sump_event_response = app.get_info("invalid-token".to_string()).await;
    assert!(sump_event_response.status().is_client_error());
}
