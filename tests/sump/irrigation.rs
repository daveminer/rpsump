// #[cfg(test)]
// mod tests {
//     use actix_web::web::Data;
//     use chrono::{Datelike, Duration, NaiveDateTime, Utc};
//     use rpsump::database::DbPool;
//     use rpsump::models::irrigation_event::{IrrigationEvent, IrrigationEventStatus};
//     use serde_json::Value;
//     use std::error::Error;
//     use std::thread::sleep;
//     use std::time::Duration as StdDuration;

//     use crate::common::fixtures::irrigation_event::insert_irrigation_event;
//     use crate::common::fixtures::irrigation_schedule::insert_irrigation_schedule;
//     use crate::common::test_app::spawn_app;
//     use crate::controllers::{create_test_user, user_params};

//     // TODO: this could fail around midnight in its current implementation.
//     #[tokio::test]
//     async fn test_irrigation_schedule() -> Result<(), Box<dyn Error>> {
//         let app = spawn_app().await;
//         let db_pool = app.db_pool.clone();

//         insert_test_data(db_pool.clone()).await;
//         let _user = create_test_user(Data::new(db_pool.clone())).await;

//         let response = app.post_login(&user_params()).await;
//         let body: Value = response.json().await.unwrap();

//         let token = body["token"].as_str().unwrap();

//         // Get the queued events
//         let response = app.get_irrigation_events(token.to_string()).await;
//         let events: Vec<IrrigationEvent> = response.json().await.unwrap();
//         let starting_queued_events: Vec<IrrigationEvent> = events
//             .into_iter()
//             .filter(|event| event.status == IrrigationEventStatus::Queued.to_string())
//             .collect();

//         assert!(starting_queued_events.len() == 0);

//         // Wait for the app to run once
//         sleep(StdDuration::from_secs(1));

//         let end_response = app.get_irrigation_events(token.to_string()).await;
//         let end_events: Vec<IrrigationEvent> = end_response.json().await.unwrap();
//         let ending_queued_events: Vec<IrrigationEvent> = end_events
//             .into_iter()
//             .filter(|event| event.status == IrrigationEventStatus::Queued.to_string())
//             .collect();

//         assert!(ending_queued_events.len() > 0);

//         Ok(())
//     }

//     async fn insert_test_data(db: DbPool) {
//         // Get the current timestamp
//         let the_past =
//             NaiveDateTime::parse_from_str("2021-01-01 12:34:56".into(), "%Y-%m-%d %H:%M:%S")
//                 .unwrap();
//         //let now = chrono::Utc::now();
//         let now = Utc::now().naive_utc();
//         let day = now.weekday();
//         let just_passed = now - Duration::seconds(3);
//         let near_future = NaiveDateTime::from(now + Duration::minutes(2));
//         let today_and_neighbors_str = format!(
//             "{},{},{}",
//             day.pred().to_string(),
//             day.to_string(),
//             day.succ().to_string()
//         );

//         // Eligible but inactive
//         insert_irrigation_schedule(
//             db.clone(),
//             false,
//             "Inactive Test Schedule".to_string(),
//             just_passed.time(),
//             15,
//             today_and_neighbors_str.clone(),
//             "1,3,4".to_string(),
//             the_past,
//         )
//         .await;

//         // Scheduled time not reached yet
//         insert_irrigation_schedule(
//             db.clone(),
//             true,
//             "Inactive Test Schedule".to_string(),
//             just_passed.time(),
//             15,
//             day.pred().to_string(),
//             "1,3,4".to_string(),
//             near_future,
//         )
//         .await;

//         // Event already ran today
//         let schedule_id = insert_irrigation_schedule(
//             db.clone(),
//             true,
//             "Inactive Test Schedule".to_string(),
//             just_passed.time(),
//             15,
//             day.pred().to_string(),
//             "1,3,4".to_string(),
//             near_future,
//         )
//         .await;

//         insert_irrigation_event(
//             db.clone(),
//             1,
//             schedule_id as i32,
//             IrrigationEventStatus::Completed.to_string(),
//             the_past,
//             Some(NaiveDateTime::from(now + Duration::seconds(15))),
//         )
//         .await;

//         // Not scheduled today
//         insert_irrigation_schedule(
//             db.clone(),
//             true,
//             "Inactive Test Schedule".to_string(),
//             just_passed.time(),
//             15,
//             day.pred().to_string(),
//             "1,3,4".to_string(),
//             the_past,
//         )
//         .await;

//         // Eligible
//         insert_irrigation_schedule(
//             db.clone(),
//             true,
//             "Eligible Test Schedule 1".to_string(),
//             just_passed.time(),
//             15,
//             today_and_neighbors_str,
//             "1,3,4".to_string(),
//             the_past,
//         )
//         .await;

//         // Also eligible
//         insert_irrigation_schedule(
//             db,
//             true,
//             "Eligible Test Schedule 2".to_string(),
//             just_passed.time(),
//             15,
//             "Monday,Tuesday,Wednesday,Thursday,Friday,Saturday,Sunday".to_string(),
//             "2".to_string(),
//             the_past,
//         )
//         .await;
//     }
// }
