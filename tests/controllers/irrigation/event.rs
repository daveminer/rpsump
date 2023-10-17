use actix_web::web::Data;
use rpsump::models::irrigation_event::IrrigationEvent;
use serde_json::Value;

use crate::common::fixtures::irrigation_event::insert_irrigation_events;
use crate::common::test_app::spawn_app;
use crate::controllers::{create_test_user, user_params};

#[tokio::test]
async fn list_events_success() {
    // Arrange
    let app = spawn_app().await;
    let db_pool = app.db_pool.clone();
    let _user = create_test_user(Data::new(db_pool.clone())).await;
    insert_irrigation_events(db_pool.clone()).await;

    // Act
    let response = app.post_login(&user_params()).await;
    let body: Value = response.json().await.unwrap();

    let token = body["token"].as_str().unwrap();

    let events_response = app.get_irrigation_events(token.to_string()).await;
    let status = events_response.status();
    let _body: Vec<IrrigationEvent> = events_response.json().await.unwrap();

    // Assert
    assert!(status.is_success());
}

