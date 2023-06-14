use actix_web::web::Data;
use serde_json::Value;

use rpsump::models::sump_event::SumpEvent;

use crate::common::test_app::spawn_app;
use crate::controllers::{create_test_user, insert_sump_events, user_params};

#[tokio::test]
async fn sump_event_success() {
    // Arrange
    let app = spawn_app().await;
    let db_pool = app.db_pool.clone();
    let _user = create_test_user(Data::new(db_pool.clone())).await;
    insert_sump_events(db_pool.clone()).await;

    // Act
    let response = app.post_login(&user_params()).await;
    let body: Value = response.json().await.unwrap();

    let token = body["token"].as_str().unwrap();

    let sump_event_response = app.get_sump_event(token.to_string()).await;
    let sump_events = sump_event_response.json::<Vec<SumpEvent>>().await.unwrap();

    // Assert
    assert!(sump_events.len() == 4);
}

#[tokio::test]
async fn sump_event_failed_no_auth() {
    let app = spawn_app().await;
    let sump_event_response = app.get_sump_event("invalid-token".to_string()).await;
    assert!(sump_event_response.status().is_client_error());
}
