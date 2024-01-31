use rpsump::repository::models::irrigation_schedule::IrrigationSchedule;

use serde_json::Value;

use crate::common::fixtures::irrigation_schedule::insert_irrigation_schedules;
use crate::common::test_app::spawn_app;
use crate::controllers::auth::create_test_user;
use crate::controllers::user_params;

#[tokio::test]
async fn get_schedule_not_found() {
    // Arrange
    let app = spawn_app().await;
    let _user = create_test_user(app.repo).await;
    insert_irrigation_schedules(app.repo, 1).await;

    // Act
    let response = app.post_login(&user_params()).await;
    let body: Value = response.json().await.unwrap();

    let token = body["token"].as_str().unwrap();

    let schedule_response = app.get_irrigation_schedule(token.to_string(), 0).await;
    let status = schedule_response.status();
    let body: Value = schedule_response.json().await.unwrap();

    // Assert
    assert!(body["message"] == "Not found");
    assert!(status == 404);
}

#[tokio::test]
async fn get_schedule_success() {
    // Arrange
    let app = spawn_app().await;
    let _user = create_test_user(app.repo).await;
    insert_irrigation_schedules(app.repo, 1).await;

    // Act
    let response = app.post_login(&user_params()).await;
    let body: Value = response.json().await.unwrap();

    let token = body["token"].as_str().unwrap();

    let schedules_response = app.get_irrigation_schedules(token.to_string()).await;
    let schedules = schedules_response
        .json::<Vec<IrrigationSchedule>>()
        .await
        .unwrap();
    let schedule_response = app
        .get_irrigation_schedule(token.to_string(), schedules[0].id)
        .await;
    let status = schedule_response.status();
    let _body: IrrigationSchedule = schedule_response.json().await.unwrap();

    // Assert
    assert!(status.is_success());
}

#[tokio::test]
async fn list_schedules_success() {
    // Arrange
    let app = spawn_app().await;
    let _user = create_test_user(app.repo).await;
    insert_irrigation_schedules(app.repo, 5).await;

    // Act
    let response = app.post_login(&user_params()).await;
    let body: Value = response.json().await.unwrap();

    let token = body["token"].as_str().unwrap();

    let events_response = app.get_irrigation_schedules(token.to_string()).await;
    let status = events_response.status();
    let _body: Vec<IrrigationSchedule> = events_response.json().await.unwrap();

    // Assert
    assert!(status.is_success());
}

#[tokio::test]
async fn delete_schedule_success() {
    // Arrange
    let app = spawn_app().await;
    let _user = create_test_user(app.repo).await;
    insert_irrigation_schedules(app.repo, 1).await;

    // Act
    let response = app.post_login(&user_params()).await;
    let body: Value = response.json().await.unwrap();

    let token = body["token"].as_str().unwrap();

    let schedule_response = app.get_irrigation_schedules(token.to_string()).await;
    let schedules = schedule_response
        .json::<Vec<IrrigationSchedule>>()
        .await
        .unwrap();
    let response = app
        .delete_irrigation_schedule(token.to_string(), schedules[0].id)
        .await;
    let status = response.status();
    let deleted_schedule: usize = response.json().await.unwrap();

    // Assert
    assert!(deleted_schedule == schedules[0].id as usize);
    assert!(status.is_success());
}

#[tokio::test]
async fn delete_schedule_not_found() {
    // Arrange
    let app = spawn_app().await;
    let _user = create_test_user(app.repo).await;

    // Act
    let response = app.post_login(&user_params()).await;
    let body: Value = response.json().await.unwrap();

    let token = body["token"].as_str().unwrap();

    let response = app.delete_irrigation_schedule(token.to_string(), 1).await;
    let status = response.status();
    let schedule_response: Value = response.json().await.unwrap();

    // Assert
    assert!(schedule_response["message"] == "Not found");
    assert!(status == 404);
}

#[tokio::test]
async fn patch_schedule_success() {
    // Arrange
    let app = spawn_app().await;
    let _user = create_test_user(app.repo).await;
    insert_irrigation_schedules(app.repo, 1).await;

    // Act
    let response = app.post_login(&user_params()).await;
    let body: Value = response.json().await.unwrap();

    let token = body["token"].as_str().unwrap();

    let schedule_response = app.get_irrigation_schedules(token.to_string()).await;
    let schedules = schedule_response
        .json::<Vec<IrrigationSchedule>>()
        .await
        .unwrap();
    let update = &schedules[0];
    let name = "Updated Name";
    let body = serde_json::json!({
        "name": name,
        "start_time": "17:34:56",
        "days_of_week": ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"]
    });
    let response = app
        .patch_irrigation_schedule(token.to_string(), update.id, body)
        .await;
    let status = response.status();
    let updated_schedule: Value = response.json().await.unwrap();

    assert!(updated_schedule["id"] == schedules[0].id);
    assert!(updated_schedule["name"] == name);
    assert!(status.is_success());
}

#[tokio::test]
async fn patch_schedule_not_found() {
    // Arrange
    let app = spawn_app().await;
    let _user = create_test_user(app.repo).await;

    // Act
    let response = app.post_login(&user_params()).await;
    let body: Value = response.json().await.unwrap();

    let token = body["token"].as_str().unwrap();

    let body = serde_json::json!({
        "name": "Updated Name",
        "start_time": "17:34:56",
        "days_of_week": ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"]
    });
    let response = app
        .patch_irrigation_schedule(token.to_string(), 1, body)
        .await;

    assert!(response.status() == 404);
}

#[tokio::test]
async fn patch_schedule_invalid() {
    // Arrange
    let app = spawn_app().await;
    let _user = create_test_user(app.repo).await;
    insert_irrigation_schedules(app.repo, 1).await;

    // Act
    let response = app.post_login(&user_params()).await;
    let body: Value = response.json().await.unwrap();

    let token = body["token"].as_str().unwrap();

    let schedule_response = app.get_irrigation_schedules(token.to_string()).await;
    let schedules = schedule_response
        .json::<Vec<IrrigationSchedule>>()
        .await
        .unwrap();
    let update = &schedules[0];

    let body = serde_json::json!({
        "start_time": "17:34123123:56",
        "days_of_week": ["Monday", "NotTuesday"]
    });
    let response = app
        .patch_irrigation_schedule(token.to_string(), update.id, body)
        .await;

    assert!(response.status().is_client_error());
}

#[tokio::test]
async fn post_schedule_success() {
    // Arrange
    let app = spawn_app().await;
    let _user = create_test_user(app.repo).await;

    // Act
    let response = app.post_login(&user_params()).await;
    let body: Value = response.json().await.unwrap();

    let token = body["token"].as_str().unwrap();

    let name = "New Schedule name";
    let body = serde_json::json!({
        "active": true,
        "hoses": [2,3,5],
        "name": name,
        "start_time": "17:34:56",
        "duration": 15,
        "days_of_week": ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"]
    });

    let schedule_response = app.post_irrigation_schedule(token.to_string(), body).await;

    let status = schedule_response.status();
    let new_schedule = schedule_response
        .json::<IrrigationSchedule>()
        .await
        .unwrap();

    assert!(new_schedule.name == name);
    assert!(status.is_success());
}

#[tokio::test]
async fn post_schedule_invalid() {
    // Arrange
    let app = spawn_app().await;
    let _user = create_test_user(app.repo).await;

    // Act
    let response = app.post_login(&user_params()).await;
    let body: Value = response.json().await.unwrap();

    let token = body["token"].as_str().unwrap();

    let name = "New Schedule name";
    let body = serde_json::json!({
        "hoses": [2,3,5],
        "name": name,
        "start_time": "27:34:56",
        "days_of_week": ["Monday"]
    });

    let schedule_response = app.post_irrigation_schedule(token.to_string(), body).await;
    let status = schedule_response.status();
    let body: Value = schedule_response.json().await.unwrap();
    assert!(body["message"] == "Json deserialize error: input is out of range at line 1 column 93");

    assert!(status.is_client_error());
}
