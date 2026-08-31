use rpsump::repository::models::user::UserFilter;
use rpsump::test_fixtures::gpio::build_mock_gpio;
use rpsump::util::ApiResponse;
use serde_json::{json, Map, Value};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockGuard, ResponseTemplate};

use super::{signup_params_with_invite, NEW_EMAIL, TEST_EMAIL, TEST_PASSWORD};
use crate::common::test_app::{spawn_app, TestApp};
use crate::controllers::user_params;

async fn mock_invite_send(app: &TestApp) -> MockGuard {
    Mock::given(path("/"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Invite email.")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await
}

async fn login_token(app: &TestApp) -> String {
    let response = app.post_login(&user_params()).await;
    let body: Value = response.json().await.unwrap();
    body["token"].as_str().unwrap().to_string()
}

fn invite_params(email: &str) -> Map<String, Value> {
    let mut map = serde_json::Map::new();
    map.insert("email".into(), email.into());
    map
}

#[tokio::test]
async fn invite_requires_authentication() {
    let app = spawn_app(&build_mock_gpio()).await;

    let response = app.post_invite_unauthenticated(&invite_params(NEW_EMAIL)).await;

    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn invite_success_creates_a_redeemable_invite() {
    let app = spawn_app(&build_mock_gpio()).await;
    let token = login_token(&app).await;
    let _mock = mock_invite_send(&app).await;

    let response = app.post_invite(token, &invite_params(NEW_EMAIL)).await;
    let status = response.status();
    let body: ApiResponse = response.json().await.unwrap();

    assert!(status.is_success());
    assert_eq!(body.message, "Invite sent.");
}

#[tokio::test]
async fn invite_rejects_an_address_that_already_has_an_account() {
    let app = spawn_app(&build_mock_gpio()).await;
    let token = login_token(&app).await;

    let response = app.post_invite(token, &invite_params(TEST_EMAIL)).await;
    let status = response.status();
    let body: ApiResponse = response.json().await.unwrap();

    assert!(status.is_client_error());
    assert_eq!(body.message, "An account with that email already exists.");
}

#[tokio::test]
async fn invite_rejects_a_malformed_address() {
    let app = spawn_app(&build_mock_gpio()).await;
    let token = login_token(&app).await;

    let response = app.post_invite(token, &invite_params("not-an-email")).await;

    assert!(response.status().is_client_error());
}

#[tokio::test]
async fn signup_rejects_an_unknown_invite_token() {
    let app = spawn_app(&build_mock_gpio()).await;

    let params = json!({
        "email": NEW_EMAIL,
        "password": TEST_PASSWORD,
        "confirm_password": TEST_PASSWORD,
        "invite_token": "no-such-token",
    });

    let response = app.post_signup(&params).await;
    let status = response.status();
    let body: ApiResponse = response.json().await.unwrap();

    assert!(status.is_client_error());
    assert_eq!(body.message, "That invitation is not valid.");
}

#[tokio::test]
async fn signup_rejects_an_invite_issued_for_a_different_address() {
    let app = spawn_app(&build_mock_gpio()).await;
    let mut params = signup_params_with_invite(&app, NEW_EMAIL).await;

    // A forwarded link must not let a third party claim the invitation.
    params["email"] = json!("someone_else@test.local");

    let response = app.post_signup(&params).await;
    let status = response.status();
    let body: ApiResponse = response.json().await.unwrap();

    assert!(status.is_client_error());
    assert_eq!(
        body.message,
        "That invitation was issued for a different email address."
    );
}

#[tokio::test]
async fn an_invite_cannot_be_redeemed_twice() {
    let app = spawn_app(&build_mock_gpio()).await;
    let params = signup_params_with_invite(&app, NEW_EMAIL).await;

    let first = app.post_signup(&params).await;
    assert!(first.status().is_success());

    let second = app.post_signup(&params).await;
    let status = second.status();
    let body: ApiResponse = second.json().await.unwrap();

    assert!(status.is_client_error());
    assert_eq!(
        body.message,
        "That invitation has expired or has already been used."
    );

    // Exactly one account exists for the invited address.
    let users = app
        .repo
        .users(UserFilter {
            email: Some(NEW_EMAIL.into()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(users.len(), 1);
}
