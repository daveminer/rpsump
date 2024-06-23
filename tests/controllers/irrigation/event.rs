use rpsump::repository::models::irrigation_event::{IrrigationEvent, IrrigationEventStatus};
use rpsump::test_fixtures::gpio::build_mock_gpio;
use serde_json::Value;

use crate::common::fixtures::irrigation_event::insert_irrigation_event;
use crate::common::fixtures::irrigation_schedule::insert_finished_schedule;
use crate::common::test_app::spawn_app;
use crate::controllers::user_params;

#[tokio::test]
async fn list_events_success() {
    // Arrange
    let app = spawn_app(&build_mock_gpio()).await;

    let sched = insert_finished_schedule(app.repo).await;

    let mut conn = app.repo.pool().await.unwrap().get().unwrap();

    insert_irrigation_event(&mut conn, 1, sched.clone(), IrrigationEventStatus::Queued).await;
    insert_irrigation_event(
        &mut conn,
        1,
        sched.clone(),
        IrrigationEventStatus::InProgress,
    )
    .await;
    insert_irrigation_event(
        &mut conn,
        1,
        sched.clone(),
        IrrigationEventStatus::Completed,
    )
    .await;

    // Act
    let response = app.post_login(&user_params()).await;
    let body: Value = response.json().await.unwrap();

    let token = body["token"].as_str().unwrap();

    let events_response = app.get_irrigation_events(token.to_string()).await;
    let status = events_response.status();
    let _body: Vec<IrrigationEvent> = events_response.json().await.unwrap();

    assert!(status.is_success());
}
