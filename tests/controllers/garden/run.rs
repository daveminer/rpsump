use rpsump::test_fixtures::gpio::build_mock_gpio;
use serde_json::{json, Value};

use crate::common::test_app::{spawn_app, TestApp};
use crate::controllers::user_params;

async fn login_token(app: &TestApp) -> String {
    let response = app.post_login(&user_params()).await;
    let body: Value = response.json().await.unwrap();
    body["token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn run_queues_a_manual_event() {
    let app = spawn_app(&build_mock_gpio()).await;
    let token = login_token(&app).await;

    let response = app
        .post_garden_run(token, json!({ "duration_secs": 30 }))
        .await;
    assert_eq!(response.status(), 202);

    let body: Value = response.json().await.unwrap();
    assert_eq!(body["source"], "manual");
    assert_eq!(body["duration_secs"], 30);
    assert!(body["schedule_id"].is_null());
    assert!(body["schedule_name"].is_null());
}

#[tokio::test]
async fn run_rejects_a_duration_over_the_hardware_limit() {
    let app = spawn_app(&build_mock_gpio()).await;
    let token = login_token(&app).await;

    // .env.test sets GARDEN_MAX_RUNTIME_SEC=60.
    let response = app
        .post_garden_run(token, json!({ "duration_secs": 61 }))
        .await;
    assert_eq!(response.status(), 400);

    let body: Value = response.json().await.unwrap();
    assert!(body["message"].as_str().unwrap().contains("60"));
}

#[tokio::test]
async fn run_rejects_a_non_positive_duration() {
    let app = spawn_app(&build_mock_gpio()).await;
    let token = login_token(&app).await;

    let response = app
        .post_garden_run(token, json!({ "duration_secs": 0 }))
        .await;
    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn a_second_run_will_not_stack_up_behind_the_first() {
    let app = spawn_app(&build_mock_gpio()).await;
    let token = login_token(&app).await;

    let first = app
        .post_garden_run(token.clone(), json!({ "duration_secs": 60 }))
        .await;
    assert_eq!(first.status(), 202);

    let second = app
        .post_garden_run(token, json!({ "duration_secs": 60 }))
        .await;
    assert_eq!(second.status(), 409);
}

#[tokio::test]
async fn stop_cancels_a_pending_run() {
    let app = spawn_app(&build_mock_gpio()).await;
    let token = login_token(&app).await;

    let run = app
        .post_garden_run(token.clone(), json!({ "duration_secs": 60 }))
        .await;
    let queued: Value = run.json().await.unwrap();
    let queued_id = queued["id"].as_i64().unwrap();

    let response = app.post_garden_stop(token.clone()).await;
    assert_eq!(response.status(), 200);

    let body: Value = response.json().await.unwrap();
    let cancelled: Vec<i64> = body["cancelled_event_ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|id| id.as_i64().unwrap())
        .collect();
    assert!(cancelled.contains(&queued_id));

    // Nothing is left pending, so another run is accepted.
    let response = app
        .post_garden_run(token, json!({ "duration_secs": 10 }))
        .await;
    assert_eq!(response.status(), 202);
}

#[tokio::test]
async fn stop_with_nothing_pending_is_a_bad_request() {
    let app = spawn_app(&build_mock_gpio()).await;
    let token = login_token(&app).await;

    let response = app.post_garden_stop(token).await;
    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn run_unauthorized() {
    let app = spawn_app(&build_mock_gpio()).await;

    let response = app
        .post_garden_run("not-a-token".to_string(), json!({ "duration_secs": 10 }))
        .await;
    assert_eq!(response.status(), 401);
}
