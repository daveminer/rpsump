use actix_web::web;
use anyhow::Error;
use chrono::{Duration, NaiveDateTime};
use diesel::{ExpressionMethods, RunQueryDsl};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockGuard, ResponseTemplate};

use rpsump::auth::token::Token;
use rpsump::controllers::ApiResponse;
use rpsump::database::{DbConn, DbPool};
use rpsump::first;
use rpsump::models::user::User;
use rpsump::schema::user;

use super::super::link_from_email_text;
use super::signup_params;
use crate::common::test_app::{spawn_app, TestApp};

#[tokio::test]
async fn email_verification_token_expired() {
    // Arrange
    let app = spawn_app().await;
    let db_pool = app.db_pool.clone();
    let params = signup_params();

    // Act
    let response = app.post_signup(&params).await;
    let status = response.status();
    assert!(status.is_success());

    let user = user_from_email(
        params["email"].as_str().unwrap().to_string(),
        db_pool.clone(),
    )
    .await
    .unwrap();

    let token_expiry = user.email_verification_token_expires_at.unwrap();
    let yesterday = token_expiry - Duration::days(1);
    let _ = set_email_verification_expiry(user.email, yesterday, db_pool.get().unwrap()).await;

    let email_verif_response = app
        .get_email_verification(user.email_verification_token.unwrap())
        .await;
    let email_verif_status = email_verif_response.status();
    let body: ApiResponse = email_verif_response.json().await.unwrap();

    // Assert
    assert!(email_verif_status.is_client_error());
    assert!(body.message == "Token expired.");
}

#[tokio::test]
async fn email_verification_failed_token_mismatch() {
    // Arrange
    let app = spawn_app().await;
    let params = signup_params();
    let token = Token::new_email_verification(0);

    // Act
    let response = app.post_signup(&params).await;
    let status = response.status();
    assert!(status.is_success());

    let email_verif_response = app.get_email_verification(token.value.to_string()).await;
    let email_verif_status = email_verif_response.status();
    let body: ApiResponse = email_verif_response.json().await.unwrap();

    // Assert
    assert!(email_verif_status.is_client_error());
    assert!(body.message == "Invalid token.");
}

#[tokio::test]
async fn email_verification_failed_no_token() {
    // Arrange
    let app = spawn_app().await;
    let params = signup_params();

    // Act
    let response = app.post_signup(&params).await;
    let status = response.status();
    assert!(status.is_success());

    let email_verif_response = app.get_email_verification("".to_string()).await;
    let email_verif_status = email_verif_response.status();
    let body: ApiResponse = email_verif_response.json().await.unwrap();

    // Assert
    assert!(email_verif_status.is_client_error());
    assert!(body.message == "Invalid token.");
}

#[tokio::test]
async fn email_verification_succeeded() {
    // Arrange
    let app = spawn_app().await;
    let params = signup_params();

    let _mock = mock_email_verification_send(&app).await;

    let response = app.post_signup(&params).await;
    let status = response.status();
    assert!(status.is_success());

    let link = email_link_from_mock_server(&app).await;

    let response = reqwest::get(link).await.unwrap();
    assert!(response.status().is_success());
    let body: ApiResponse = response.json().await.unwrap();

    assert!(body.message == "Email verified.");
}

async fn set_email_verification_expiry(
    email: String,
    time: NaiveDateTime,
    mut conn: DbConn,
) -> Result<usize, anyhow::Error> {
    diesel::update(user::table)
        .filter(user::email.eq(email))
        .set(user::email_verification_token_expires_at.eq(time.to_string()))
        .execute(&mut conn)
        .map_err(|e| anyhow::Error::new(e))
}

async fn user_from_email(email: String, db_pool: DbPool) -> Result<User, Error> {
    let user = first!(User::by_email(email), User, db_pool).unwrap();
    Ok(user)
}

async fn email_link_from_mock_server(app: &TestApp) -> String {
    let verification_email = app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();
    let body = std::str::from_utf8(&verification_email.body).unwrap();

    let link = link_from_email_text(body);

    link[0].clone()
}

async fn mock_email_verification_send(app: &TestApp) -> MockGuard {
    Mock::given(path("/"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Email verification.")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await
}
