use rpsump::test_fixtures::gpio::build_mock_gpio;
use serde_json::{json, Value};

use crate::common::test_app::{spawn_app, TestApp};
use crate::controllers::user_params;

async fn login_token(app: &TestApp) -> String {
    let response = app.post_login(&user_params()).await;
    let body: Value = response.json().await.unwrap();
    body["token"].as_str().unwrap().to_string()
}

async fn events(app: &TestApp, token: &str, query: &str) -> Vec<Value> {
    let response = app
        .get_garden_events_with_query(token.to_string(), query)
        .await;
    assert_eq!(response.status(), 200);

    let body: Value = response.json().await.unwrap();
    body.as_array().unwrap().clone()
}

#[tokio::test]
async fn events_can_be_filtered_by_source() {
    let app = spawn_app(&build_mock_gpio()).await;
    let token = login_token(&app).await;

    let response = app
        .post_garden_run(token.clone(), json!({ "duration_secs": 60 }))
        .await;
    assert_eq!(response.status(), 202);

    let manual = events(&app, &token, "?source=manual").await;
    assert_eq!(manual.len(), 1);
    assert_eq!(manual[0]["source"], "manual");

    let scheduled = events(&app, &token, "?source=scheduled").await;
    assert!(scheduled.is_empty());
}

#[tokio::test]
async fn events_can_be_filtered_by_time_window() {
    let app = spawn_app(&build_mock_gpio()).await;
    let token = login_token(&app).await;

    let _ = app
        .post_garden_run(token.clone(), json!({ "duration_secs": 60 }))
        .await;

    let in_window = events(&app, &token, "?from=2020-01-01T00:00:00Z").await;
    assert_eq!(in_window.len(), 1);

    let before_window = events(&app, &token, "?to=2020-01-01T00:00:00Z").await;
    assert!(before_window.is_empty());
}

#[tokio::test]
async fn events_reject_unparseable_filters() {
    let app = spawn_app(&build_mock_gpio()).await;
    let token = login_token(&app).await;

    for query in [
        "?status=sideways",
        "?source=whenever",
        "?from=yesterday",
        "?from=2026-08-29T00:00:00Z&to=2026-08-01T00:00:00Z",
    ] {
        let response = app
            .get_garden_events_with_query(token.clone(), query)
            .await;
        assert_eq!(response.status(), 400, "expected 400 for {}", query);
    }
}

#[tokio::test]
async fn events_can_be_filtered_by_status() {
    let app = spawn_app(&build_mock_gpio()).await;
    let token = login_token(&app).await;

    let _ = app
        .post_garden_run(token.clone(), json!({ "duration_secs": 60 }))
        .await;

    let skipped = events(&app, &token, "?status=skipped").await;
    assert!(skipped.is_empty());
}
