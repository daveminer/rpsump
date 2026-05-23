use rpsump::test_fixtures::gpio::build_mock_gpio;
use serde_json::{json, Value};

use crate::common::fixtures::garden::{schedule_params, schedule_params_named};
use crate::common::test_app::spawn_app;
use crate::controllers::user_params;

async fn login_token(app: &crate::common::test_app::TestApp) -> String {
    let response = app.post_login(&user_params()).await;
    let body: Value = response.json().await.unwrap();
    body["token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn create_schedule_success() {
    let app = spawn_app(&build_mock_gpio()).await;
    let token = login_token(&app).await;

    let response = app.post_garden_schedule(token, schedule_params()).await;
    assert_eq!(response.status(), 200);

    let body: Value = response.json().await.unwrap();
    assert_eq!(body["name"], "Test Schedule");
    assert_eq!(body["duration_secs"], 30);
    assert!(body["start_times"].is_array());
    assert!(body["days_of_week"].is_array());
}

#[tokio::test]
async fn create_schedule_rejects_empty_times() {
    let app = spawn_app(&build_mock_gpio()).await;
    let token = login_token(&app).await;

    let bad = json!({
        "name": "Empty",
        "active": true,
        "start_times": [],
        "days_of_week": ["Mon"],
        "duration_secs": 10
    });
    let response = app.post_garden_schedule(token, bad).await;
    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn list_schedules_returns_created_schedules() {
    let app = spawn_app(&build_mock_gpio()).await;
    let token = login_token(&app).await;

    let _ = app
        .post_garden_schedule(token.clone(), schedule_params_named("A"))
        .await;
    let _ = app
        .post_garden_schedule(token.clone(), schedule_params_named("B"))
        .await;

    let response = app.get_garden_schedules(token).await;
    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();
    let arr = body.as_array().unwrap();
    assert!(arr.len() >= 2);
}

#[tokio::test]
async fn get_schedule_not_found() {
    let app = spawn_app(&build_mock_gpio()).await;
    let token = login_token(&app).await;

    let response = app.get_garden_schedule(token, 99999).await;
    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn update_schedule_success() {
    let app = spawn_app(&build_mock_gpio()).await;
    let token = login_token(&app).await;

    let create = app
        .post_garden_schedule(token.clone(), schedule_params())
        .await;
    let body: Value = create.json().await.unwrap();
    let id = body["id"].as_i64().unwrap() as i32;

    let patch_body = json!({ "active": false, "duration_secs": 60 });
    let response = app.patch_garden_schedule(token, id, patch_body).await;
    assert_eq!(response.status(), 200);

    let body: Value = response.json().await.unwrap();
    assert_eq!(body["active"], false);
    assert_eq!(body["duration_secs"], 60);
}

#[tokio::test]
async fn delete_schedule_success() {
    let app = spawn_app(&build_mock_gpio()).await;
    let token = login_token(&app).await;

    let create = app
        .post_garden_schedule(token.clone(), schedule_params())
        .await;
    let body: Value = create.json().await.unwrap();
    let id = body["id"].as_i64().unwrap() as i32;

    let response = app.delete_garden_schedule(token.clone(), id).await;
    assert_eq!(response.status(), 200);

    let response = app.get_garden_schedule(token, id).await;
    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn schedule_unauthorized() {
    let app = spawn_app(&build_mock_gpio()).await;
    let response = app
        .get_garden_schedules("not-a-token".to_string())
        .await;
    assert_eq!(response.status(), 401);
}
