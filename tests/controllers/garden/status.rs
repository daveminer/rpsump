use rpsump::test_fixtures::gpio::build_mock_gpio;
use serde_json::Value;

use crate::common::test_app::spawn_app;
use crate::controllers::user_params;

#[tokio::test]
async fn status_endpoint_returns_expected_shape() {
    let app = spawn_app(&build_mock_gpio()).await;

    let response = app.post_login(&user_params()).await;
    let body: Value = response.json().await.unwrap();
    let token = body["token"].as_str().unwrap().to_string();

    let response = app.get_garden_status(token).await;
    assert_eq!(response.status(), 200);

    let body: Value = response.json().await.unwrap();
    assert!(body.get("is_on").is_some());
    assert!(body.get("current_event").is_some());
    assert!(body.get("next_run_at").is_some());
    assert_eq!(body["is_on"], false);
    assert!(body["current_event"].is_null());

    // The Irrigation screen reads its rolling totals and its slider bound
    // straight off this response.
    assert_eq!(body["watering_totals"]["last_24h_secs"], 0);
    assert_eq!(body["watering_totals"]["last_3d_secs"], 0);
    assert_eq!(body["watering_totals"]["last_7d_secs"], 0);
    assert_eq!(body["rain_skips_48h"], 0);
    assert_eq!(body["max_runtime_secs"], 60);
}
