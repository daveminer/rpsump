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
async fn report_returns_a_bucket_per_day_in_the_window() {
    let app = spawn_app(&build_mock_gpio()).await;
    let token = login_token(&app).await;

    let response = app.get_garden_report(token, "").await;
    assert_eq!(response.status(), 200);

    let body: Value = response.json().await.unwrap();
    assert_eq!(body["days"], 14);
    assert_eq!(body["daily"].as_array().unwrap().len(), 14);
    assert!(body["totals"]["scheduled"].is_object());
    assert!(body["totals"]["manual"].is_object());
    assert!(body["by_schedule"].is_array());
    assert!(body["outcomes"]["completed"].is_i64());
    assert_eq!(body["rain_skips"]["runs"], 0);
}

#[tokio::test]
async fn report_window_is_configurable_and_clamped() {
    let app = spawn_app(&build_mock_gpio()).await;
    let token = login_token(&app).await;

    let response = app.get_garden_report(token.clone(), "?days=3").await;
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["days"], 3);

    let response = app.get_garden_report(token, "?days=5000").await;
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["days"], 90);
}

#[tokio::test]
async fn report_counts_a_manual_run() {
    let app = spawn_app(&build_mock_gpio()).await;
    let token = login_token(&app).await;

    let _ = app
        .post_garden_run(token.clone(), json!({ "duration_secs": 60 }))
        .await;

    let response = app.get_garden_report(token, "").await;
    let body: Value = response.json().await.unwrap();

    // The run may already have been picked up by the scheduler, so it counts
    // as either queued or in progress.
    let outcomes = &body["outcomes"];
    assert_eq!(
        outcomes["queued"].as_i64().unwrap() + outcomes["in_progress"].as_i64().unwrap(),
        1
    );
    // Manual runs are reported in totals, never attributed to a schedule.
    assert!(body["by_schedule"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn report_unauthorized() {
    let app = spawn_app(&build_mock_gpio()).await;

    let response = app.get_garden_report("not-a-token".to_string(), "").await;
    assert_eq!(response.status(), 401);
}
