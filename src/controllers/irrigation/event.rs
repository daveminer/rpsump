// use actix_web::{error, get, web::Data, HttpResponse, Responder, Result};
// use anyhow::{anyhow, Error};
// use diesel::{QueryDsl, RunQueryDsl};

// use crate::auth::authenticated_user::AuthenticatedUser;
// use crate::controllers::spawn_blocking_with_tracing;
// use crate::database::{self, DbPool};
// use crate::models::irrigation_event::IrrigationEvent;

// #[get("/irrigation_event")]
// #[tracing::instrument(skip(_req_body, db, _user))]
// pub async fn irrigation_event(
//     _req_body: String,
//     db: Data<DbPool>,
//     _user: AuthenticatedUser,
// ) -> Result<impl Responder> {
//     let irrigation_events = spawn_blocking_with_tracing(move || irrigation_events(db))
//         .await
//         .map_err(|e| {
//             tracing::error!("Error while spawning a blocking task: {:?}", e);
//             error::ErrorInternalServerError("Internal server error.")
//         })?
//         .map_err(|e| {
//             tracing::error!("Error while getting sump events: {:?}", e);
//             error::ErrorInternalServerError("Internal server error.")
//         })?;

//     Ok(HttpResponse::Ok().json(irrigation_events))
// }

// fn irrigation_events(db: Data<DbPool>) -> Result<Vec<IrrigationEvent>, Error> {
//     let mut conn = database::conn(db)?;
//     let irrigation_events: Vec<IrrigationEvent> = IrrigationEvent::all()
//         .limit(100)
//         .load::<IrrigationEvent>(&mut conn)
//         .map_err(|e| anyhow!(e))?;

//     Ok(irrigation_events)
// }
