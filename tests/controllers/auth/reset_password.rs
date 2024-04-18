use chrono::{Duration, Utc};
use rpsump::test_fixtures::gpio::build_mock_gpio;
use rpsump::util::ApiResponse;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

use rpsump::auth::token::Token;
use rpsump::repository::models::user::{User, UserFilter, UserUpdateFilter};

use super::signup_params;
use crate::common::test_app::{spawn_app, TestApp};
use crate::controllers::{auth::password_reset_params, param_from_email_text};

const NEW_PASSWORD: &str = "new_%Password53";
const INVALID_NEW_PASSWORD: &str = "123";

#[tokio::test]
async fn reset_password_failed_invalid_token() {
    let (app, _params) = signup_and_request_password_reset().await;

    // Validate the reset password response
    let reset_response = app
        .post_password_reset(&password_reset_params(
            "wrong_token".into(),
            NEW_PASSWORD.into(),
        ))
        .await;

    assert!(reset_response.status().is_client_error());
    let reset_body: ApiResponse = reset_response.json().await.unwrap();
    assert!(reset_body.message == "Invalid token.");
}

#[tokio::test]
async fn reset_password_failed_invalid_password() {
    let (app, _params) = signup_and_request_password_reset().await;
    let token = get_token_from_email(&app).await;

    // Validate the reset password response
    let reset_response = app
        .post_password_reset(&password_reset_params(token, INVALID_NEW_PASSWORD.into()))
        .await;

    assert!(reset_response.status().is_client_error());
    let reset_body: ApiResponse = reset_response.json().await.unwrap();
    assert!(reset_body.message == "Validation error: Password is too short. [{}]");
}

#[tokio::test]
async fn reset_password_failed_token_expired() {
    let (app, _params) = signup_and_request_password_reset().await;
    let token = get_token_from_email(&app).await;

    let filter = UserFilter {
        id: None,
        email: None,
        email_verification_token: None,
        password_hash: None,
        password_reset_token: Some(token.clone()),
    };
    let user: User = app
        .repo
        .users(filter)
        .await
        .unwrap()
        .first()
        .unwrap()
        .clone();

    let expired_token = Token {
        user_id: user.id,
        expires_at: Utc::now().naive_utc() - Duration::hours(1),
        value: token.clone(),
    };

    let yesterday = Utc::now().naive_utc() - Duration::days(1);

    let user_update_filter = UserUpdateFilter {
        id: user.id,
        email: None,
        email_verification_token: None,
        email_verification_token_expires_at: None,
        password_hash: None,
        password_reset_token: Some(Some(expired_token.value.clone())),
        password_reset_token_expires_at: Some(yesterday),
    };
    let _user_update = app.repo.update_user(user_update_filter).await.unwrap();

    // Validate the reset password response
    let reset_response = app
        .post_password_reset(&password_reset_params(token, NEW_PASSWORD.into()))
        .await;

    assert!(reset_response.status().is_client_error());
    let reset_body: ApiResponse = reset_response.json().await.unwrap();
    assert!(reset_body.message == "Token expired.");
}

#[tokio::test]
async fn reset_password_success() {
    let (app, params) = signup_and_request_password_reset().await;
    let token = get_token_from_email(&app).await;

    // Validate the reset password response
    let reset_response = app
        .post_password_reset(&password_reset_params(token, NEW_PASSWORD.into()))
        .await;

    assert!(reset_response.status().is_success());
    let reset_body: ApiResponse = reset_response.json().await.unwrap();
    assert!(reset_body.message == "Password reset successfully.");

    // Try to log in with the old password
    let stale_login_response = app.post_login(&params).await;
    assert!(stale_login_response.status().is_client_error());

    // Log in with the new password
    let mut new_login_params = serde_json::Map::new();
    new_login_params.insert("email".into(), params["email"].as_str().unwrap().into());
    new_login_params.insert("password".into(), NEW_PASSWORD.into());

    let new_login_response = app.post_login(&new_login_params).await;
    assert!(new_login_response.status().is_success());
}

async fn signup_and_request_password_reset() -> (TestApp, serde_json::Map<String, serde_json::Value>)
{
    let app = spawn_app(build_mock_gpio).await;
    let params = signup_params();

    // Mock the email verification and reset password email sends.
    let _mock_guard = Mock::given(path("/"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Email sends.")
        .expect(2)
        .mount_as_scoped(&app.email_server)
        .await;

    // Signup
    let response = app.post_signup(&params).await;
    assert!(response.status().is_success());

    // Request a password reset
    let email = params["email"].as_str().unwrap().to_string();
    let mut map = serde_json::Map::new();
    map.insert("email".into(), email.clone().into());
    let reset_password_response = app.post_request_password_reset(&map).await;
    assert!(reset_password_response.status().is_success());

    (app, params)
}

async fn get_token_from_email(app: &TestApp) -> String {
    let mut reqs = app.email_server.received_requests().await.unwrap();

    let pw_confirm = reqs.pop().unwrap();

    let body = std::str::from_utf8(&pw_confirm.body).unwrap();
    let params = param_from_email_text(body, "token");
    params[0].clone()
}
