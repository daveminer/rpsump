use reqwest::header::{
    ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN, ACCESS_CONTROL_REQUEST_METHOD,
    ORIGIN,
};
use rpsump::test_fixtures::gpio::build_mock_gpio;

use crate::common::test_app::{spawn_app, TestApp};

async fn preflight(app: &TestApp, origin: &str, method: &str) -> reqwest::Response {
    app.api_client
        .request(
            reqwest::Method::OPTIONS,
            &format!("{}/garden/schedule/1", &app.address),
        )
        .header(ORIGIN, origin)
        .header(ACCESS_CONTROL_REQUEST_METHOD, method)
        .send()
        .await
        .unwrap()
}

#[tokio::test]
async fn the_browser_may_edit_and_delete_schedules() {
    let app = spawn_app(&build_mock_gpio()).await;

    for method in ["PATCH", "DELETE"] {
        let response = preflight(&app, "http://localhost:5173", method).await;
        assert_eq!(response.status(), 200, "preflight rejected for {}", method);

        let allowed = response
            .headers()
            .get(ACCESS_CONTROL_ALLOW_METHODS)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        assert!(allowed.contains(method), "{} not in {}", method, allowed);
    }
}

#[tokio::test]
async fn a_configured_origin_is_allowed() {
    let app = spawn_app(&build_mock_gpio()).await;

    // .env.test sets SERVER_ALLOWED_ORIGINS to this origin.
    let response = preflight(&app, "https://nimbus.example.com", "PATCH").await;
    assert_eq!(response.status(), 200);
    assert_eq!(
        response
            .headers()
            .get(ACCESS_CONTROL_ALLOW_ORIGIN)
            .unwrap()
            .to_str()
            .unwrap(),
        "https://nimbus.example.com"
    );
}

#[tokio::test]
async fn an_unlisted_origin_is_refused() {
    let app = spawn_app(&build_mock_gpio()).await;

    for origin in ["https://localhost.attacker.com", "https://not-nimbus.com"] {
        let response = preflight(&app, origin, "PATCH").await;
        assert!(
            response.headers().get(ACCESS_CONTROL_ALLOW_ORIGIN).is_none(),
            "{} was allowed",
            origin
        );
    }
}
