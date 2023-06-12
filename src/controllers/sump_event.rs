use actix_web::{error, get, web::Data, HttpResponse, Responder, Result};
use anyhow::{anyhow, Error};
use diesel::{QueryDsl, RunQueryDsl};

use crate::auth::authenticated_user::AuthenticatedUser;
use crate::controllers::spawn_blocking_with_tracing;
use crate::database::{self, DbPool};
use crate::models::sump_event::SumpEvent;

#[get("/sump_event")]
#[tracing::instrument(skip(_req_body, db, _user))]
pub async fn sump_event(
    _req_body: String,
    db: Data<DbPool>,
    // TODO: check if needed
    _user: AuthenticatedUser,
) -> Result<impl Responder> {
    let sump_events = spawn_blocking_with_tracing(move || sump_events(db))
        .await
        .map_err(|e| {
            tracing::error!("Error while spawning a blocking task: {:?}", e);
            error::ErrorInternalServerError("Internal server error.")
        })?
        .map_err(|e| {
            tracing::error!("Error while getting sump events: {:?}", e);
            error::ErrorInternalServerError("Internal server error.")
        })?;

    Ok(HttpResponse::Ok().json(sump_events))
}

fn sump_events(db: Data<DbPool>) -> Result<Vec<SumpEvent>, Error> {
    let mut conn = database::conn(db)?;
    let sump_events: Vec<SumpEvent> = SumpEvent::all()
        .limit(100)
        .load::<SumpEvent>(&mut conn)
        .map_err(|e| anyhow!(e))?;

    Ok(sump_events)
}
