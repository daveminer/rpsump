use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

use rpsump::controllers::ApiResponse;

use super::signup_params;
use crate::common::test_app::spawn_app;
use crate::controllers::{auth::password_reset_params, param_from_email_text};

#[tokio::test]
async fn reset_password_success() {
    let app = spawn_app().await;
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

    // Use the email to reset the password
    let email = params["email"].as_str().unwrap().to_string();
    let mut map = serde_json::Map::new();
    map.insert("email".into(), email.clone().into());
    let reset_password_response = app.post_request_password_reset(&map).await;
    assert!(reset_password_response.status().is_success());

    // Get the token from the email
    let reset_password_email = app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();
    let body = std::str::from_utf8(&reset_password_email.body).unwrap();
    let token = param_from_email_text(body, "token");

    // Validate the reset password response
    let new_password = "new_%Password53";
    let reset_response = app
        .post_password_reset(&password_reset_params(
            token[0].clone(),
            new_password.into(),
        ))
        .await;
    assert!(reset_response.status().is_success());
    let reset_body: ApiResponse = reset_response.json().await.unwrap();
    assert!(reset_body.message == "Password reset successfully.");

    // Try to log in with the old password
    let stale_login_response = app.post_login(&params).await;
    assert!(stale_login_response.status().is_client_error());

    // Log in with the new password
    let mut new_login_params = serde_json::Map::new();
    new_login_params.insert("email".into(), email.into());
    new_login_params.insert("password".into(), new_password.into());

    let new_login_response = app.post_login(&new_login_params).await;
    assert!(new_login_response.status().is_success());
}
