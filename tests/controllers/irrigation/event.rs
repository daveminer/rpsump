use actix_web::web::Data;
use anyhow::Error;
use rpsump::models::user_event::UserEvent;
use serde_json::Value;

use rpsump::controllers::ApiResponse;
use rpsump::database::DbPool;
use rpsump::models::user::User;
use rpsump::models::user_event::EventType;

use super::{create_test_user, user_params};
use crate::common::test_app::spawn_app;

#[tokio::test]
async fn event_success() {
    // Arrange
    let app = spawn_app().await;
    let db_pool = app.db_pool.clone();
    //let user = create_test_user(Data::new(db_pool.clone())).await;

    // Act
    let response = app.get_irrigation_event().await;
    let status = response.status();
    let body: Value = response.json().await.unwrap();

    // Assert
    assert!(status.is_success());
    //assert!(body["token"].is_string());
    //let events = recent_login_events(user.clone(), db_pool).await.unwrap();
    //assert_eq!(events.len(), 1);
}

// async fn recent_login_events(record: User, db_pool: DbPool) -> Result<Vec<UserEvent>, Error> {
//     UserEvent::recent_events(
//         Some(record),
//         None,
//         EventType::Login,
//         10,
//         actix_web::web::Data::new(db_pool),
//     )
//     .await
// }
