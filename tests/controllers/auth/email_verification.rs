use chrono::{Duration, NaiveDateTime};
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::{ExpressionMethods, RunQueryDsl, SqliteConnection};

use rpsump::repository::models::user::{User, UserFilter};
use rpsump::test_fixtures::gpio::build_mock_gpio;
use rpsump::{auth::token::Token, schema::user, util::ApiResponse};

use super::{NEW_EMAIL, TEST_PASSWORD};
use crate::common::test_app::{spawn_app, TestApp};

/// Signup no longer produces verification tokens: redeeming an invite proves
/// address control, so invited accounts are verified outright. These tests
/// exercise the `verify_email` endpoint itself, so they create the user and
/// mint the token directly rather than going through signup.
async fn user_awaiting_verification(app: &TestApp) -> User {
    let created = app
        .repo
        .create_user(
            NEW_EMAIL.to_string(),
            TEST_PASSWORD.to_string(),
            "127.0.0.1".to_string(),
        )
        .await
        .unwrap();

    let _token = app.repo.create_email_verification(&created).await.unwrap();

    // Re-read so the token columns written by create_email_verification are
    // present on the returned record.
    app.repo
        .users(UserFilter {
            email: Some(NEW_EMAIL.into()),
            ..Default::default()
        })
        .await
        .unwrap()
        .pop()
        .unwrap()
}

#[tokio::test]
async fn email_verification_token_expired() {
    // Arrange
    let app = spawn_app(&build_mock_gpio()).await;
    let user = user_awaiting_verification(&app).await;

    let token_expiry = user.email_verification_token_expires_at.unwrap();
    let yesterday = token_expiry - Duration::days(1);
    let db = app.repo.pool().await.unwrap().get().unwrap();
    let _ = set_email_verification_expiry(user.email, yesterday, db).await;

    // Act
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
    let app = spawn_app(&build_mock_gpio()).await;
    let _user = user_awaiting_verification(&app).await;
    let token = Token::new_email_verification(0);

    // Act
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
    let app = spawn_app(&build_mock_gpio()).await;
    let _user = user_awaiting_verification(&app).await;

    // Act
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
    let app = spawn_app(&build_mock_gpio()).await;
    let user = user_awaiting_verification(&app).await;

    // Act
    let response = app
        .get_email_verification(user.email_verification_token.unwrap())
        .await;
    let status = response.status();
    let body: ApiResponse = response.json().await.unwrap();

    // Assert
    assert!(status.is_success());
    assert!(body.message == "Email verified.");

    let verified = app
        .repo
        .users(UserFilter {
            email: Some(NEW_EMAIL.into()),
            ..Default::default()
        })
        .await
        .unwrap()
        .pop()
        .unwrap();
    assert!(verified.email_verified_at.is_some());
}

async fn set_email_verification_expiry(
    email: String,
    time: NaiveDateTime,
    mut conn: PooledConnection<ConnectionManager<SqliteConnection>>,
) -> Result<usize, anyhow::Error> {
    diesel::update(user::table)
        .filter(user::email.eq(email))
        .set(user::email_verification_token_expires_at.eq(time.to_string()))
        .execute(&mut conn)
        .map_err(anyhow::Error::new)
}
