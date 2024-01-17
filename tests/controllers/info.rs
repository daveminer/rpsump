use actix_web::web::Data;
use serde_json::Value;

use crate::common::fixtures::sump_event::insert_sump_events;
use crate::common::test_app::spawn_app;
use crate::controllers::{create_test_user, user_params};

#[tokio::test]
async fn info_success_sump_disabled() {
    // Arrange
    let app = spawn_app().await;
    let db_pool = app.db_pool.clone();
    let _user = create_test_user(Data::new(db_pool.clone())).await;
    let _sump_events = insert_sump_events(db_pool.clone()).await;

    // Act
    let response = app.post_login(&user_params()).await;
    let body: Value = response.json().await.unwrap();
    let token = body["token"].as_str().unwrap();

    let sump_event_response = app.get_info(token.to_string()).await;
    let response: Value = sump_event_response.json().await.unwrap();

    // Assert
    assert!(response["heater"].as_bool() == Some(false));
}

#[tokio::test]
async fn info_failed_no_auth() {
    let app = spawn_app().await;
    let sump_event_response = app.get_info("invalid-token".to_string()).await;
    assert!(sump_event_response.status().is_client_error());
}
