use chrono::{Duration, NaiveDateTime};
use diesel::{ExpressionMethods, RunQueryDsl};

use rpsump::auth::token::Token;
use rpsump::controllers::ApiResponse;
use rpsump::models::user::User;
use rpsump::repository::DbConn;
use rpsump::schema::user;

use super::signup_params;
use crate::common::test_app::spawn_app;
use crate::controllers::{email_link_from_mock_server, mock_email_verification_send};

#[tokio::test]
async fn email_verification_token_expired() {
    // Arrange
    let app = spawn_app().await;
    let mut db = app.db_pool.get().unwrap();
    let params = signup_params();
    let _mock = mock_email_verification_send(&app).await;

    // Act
    let response = app.post_signup(&params).await;
    let status = response.status();
    assert!(status.is_success());

    let user: User = User::by_email(params["email"].as_str().unwrap().to_string())
        .first(&mut db)
        .unwrap();

    let token_expiry = user.email_verification_token_expires_at.unwrap();
    let yesterday = token_expiry - Duration::days(1);
    let _ = set_email_verification_expiry(user.email, yesterday, db).await;

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
    let _mock = mock_email_verification_send(&app).await;

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
    let _mock = mock_email_verification_send(&app).await;

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
