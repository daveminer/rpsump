use rpsump::test_fixtures::gpio::mock_gpio_get;
use serde_json::Value;

use crate::common::test_app::{spawn_app, spawn_app_with_gpio};
use crate::controllers::auth::create_test_user;
use crate::controllers::user_params;

#[tokio::test]
async fn heater_success() {
    // Arrange
    // TODO: fix this
    let gpio = mock_gpio_get(vec![
        1, 1, 7, 7, 8, 8, 14, 14, 15, 15, 17, 17, 18, 18, 22, 22, 23, 23, 24, 24, 25, 25, 26, 26,
        27, 27, 32, 32,
    ]);

    let app = spawn_app_with_gpio(&gpio).await;
    let _user = create_test_user(app.repo).await;

    // Act
    let response = app.post_login(&user_params()).await;
    let body: Value = response.json().await.unwrap();
    let token = body["token"].as_str().unwrap();

    let on_response = app.post_heater_on(token.to_string()).await;
    let on_response_body: Value = on_response.json().await.unwrap();

    let off_response = app.post_heater_off(token.to_string()).await;
    let off_response_body: Value = off_response.json().await.unwrap();

    // Assert
    assert_eq!(on_response_body["status"].as_str(), Some("ok"));
    assert_eq!(off_response_body["status"].as_str(), Some("ok"))
}

#[tokio::test]
async fn heater_failed_no_auth() {
    // Arrange
    let app = spawn_app().await;

    let sump_event_response = app.post_heater_on("123".to_string()).await;

    // Assert
    assert!(sump_event_response.status() == 401);
}
