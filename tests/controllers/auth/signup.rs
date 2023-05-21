use actix_web::web;
use anyhow::Error;
use diesel::RunQueryDsl;

use rpsump::database::DbPool;
use rpsump::first;
use rpsump::models::user::User;
use rpsump::models::user_event::{EventType, UserEvent};
use serde_json::Value;

use super::{signup_params, user_params};
use crate::common::test_app::spawn_app;

//TODO: cover identity creation

//#[tokio::test]
// async fn login_failed_username_not_found() {
//     // Arrange
//     let app = spawn_app().await;

//     // Act
//     let response = app.post_login(&user_params()).await;
//     let status = response.status();
//     let body: ErrorBody = response.json().await.unwrap();

//     // Assert
//     assert!(status.is_client_error());
//     assert_eq!(body.reason, "Invalid email or password.");
// }
// #[tokio::test]
// async fn login_password_incorrect() {
//     // Arrange
//     let app = spawn_app().await;
//     let db_pool = app.db_pool.clone();
//     let _user = create_test_user(Data::new(db_pool.clone())).await;
//     let mut params = user_params();
//     params["password"] = "wrong_password".into();

//     // Act
//     let response = app.post_login(&params).await;
//     let status = response.status();
//     let body: ErrorBody = response.json().await.unwrap();

//     // Assert
//     assert!(status.is_client_error());
//     assert_eq!(body.reason, "Invalid email or password.");
// }

// #[tokio::test]
// async fn login_missing_email() {
//     // Arrange
//     let app = spawn_app().await;
//     let db_pool = app.db_pool.clone();
//     let _user = create_test_user(Data::new(db_pool.clone())).await;
//     let mut params = user_params();
//     params["email"] = "".into();

//     // Act
//     let response = app.post_login(&params).await;
//     let status = response.status();
//     let body: ErrorBody = response.json().await.unwrap();

//     // Assert
//     assert!(status.is_client_error());
//     assert_eq!(body.reason, "Email and password are required.");
// }

#[tokio::test]
async fn signup_failed_missing_confirm_password() {
    // Arrange
    let app = spawn_app().await;
    let params = user_params();
    // params["password"] = "".into();

    // Act
    let response = app.post_signup(&params).await;
    let status = response.status();
    println!("REPPP {:?}", response.text().await);
    //let body: ErrorBody = response.json().await.unwrap();

    // Assert
    assert!(status.is_client_error());
    //assert_eq!(body.reason, "Email and password are required.");
}

#[tokio::test]
async fn signup_success() {
    // Arrange
    let app = spawn_app().await;
    let db_pool = app.db_pool.clone();
    let params = signup_params();
    let email = params.get("email").unwrap().as_str().unwrap();

    // Act
    let response = app.post_signup(&params).await;
    let status = response.status();
    let body: Value = response.json().await.unwrap();

    // Assert
    assert!(status.is_success());
    assert_eq!(body["message"], "User created.");

    let events = recent_signup_events(email.to_string(), db_pool)
        .await
        .unwrap();
    assert_eq!(events.len(), 1);
}

async fn recent_signup_events(email: String, db_pool: DbPool) -> Result<Vec<UserEvent>, Error> {
    let db_clone = db_pool.clone();
    let user = first!(User::by_email(email), User, &db_pool).unwrap();

    UserEvent::recent_events(
        Some(user),
        None,
        EventType::Signup,
        10,
        actix_web::web::Data::new(db_clone),
    )
    .await
}
