use actix_web::web;
use anyhow::Error;
use chrono::NaiveDateTime;
use chrono::{DateTime, Duration, Utc};

use diesel::ExpressionMethods;
use diesel::RunQueryDsl;

use rpsump::auth::token::Token;
use rpsump::controllers::ApiResponse;
use rpsump::database::{DbConn, DbPool};
use rpsump::first;
use rpsump::models::user::User;
use rpsump::schema::user;

use super::signup_params;
use crate::common::test_app::spawn_app;

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

// TODO: test email
// #[tokio::test]
// async fn email_verification_succeeded() {
//     // Arrange
//     let app = spawn_app().await;
//     let db_pool = app.db_pool.clone();
//     let params = signup_params();

//     // Act
//     let response = app.post_signup(&params).await;
//     let status = response.status();
//     assert!(status.is_success());

//     let token = token_for_email(params["email"].as_str().unwrap().to_string(), db_pool)
//         .await
//         .unwrap();

//     let email_verif_response = app.get_email_verification(token).await;
//     let email_verif_status = email_verif_response.status();
//     let body: ApiResponse = email_verif_response.json().await.unwrap();

//     // Assert
//     assert!(email_verif_status.is_success());
//     assert!(body.message == "Email verified.");
// }

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

async fn token_for_email(email: String, db_pool: DbPool) -> Result<String, Error> {
    let user = first!(User::by_email(email), User, db_pool).unwrap();
    Ok(user.email_verification_token.unwrap())
}

async fn user_from_email(email: String, db_pool: DbPool) -> Result<User, Error> {
    let user = first!(User::by_email(email), User, db_pool).unwrap();
    Ok(user)
}
