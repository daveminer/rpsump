use actix_web::{error, get, web::Data, HttpResponse, Responder, Result};
use anyhow::{anyhow, Error};
use diesel::{QueryDsl, RunQueryDsl};

use crate::auth::authenticated_user::AuthenticatedUser;
use crate::database::DbPool;
use crate::models::sump_event::SumpEvent;
use crate::util::spawn_blocking_with_tracing;

#[get("/sump_event")]
#[tracing::instrument(skip(_req_body, db, _user))]
pub async fn sump_event(
    _req_body: String,
    db: Data<dyn DbPool>,
    _user: AuthenticatedUser,
) -> Result<HttpResponse> {
    let sump_events = spawn_blocking_with_tracing(move || sump_events(db))
        .await
        .map_err(|e| {
            tracing::error!(
                target = module_path!(),
                error = e.to_string(),
                "Error while spawning a blocking task"
            );
            error::ErrorInternalServerError("Internal server error.")
        })?
        .map_err(|e| {
            tracing::error!(
                target = module_path!(),
                error = e.to_string(),
                "Error while getting sump events"
            );
            error::ErrorInternalServerError("Internal server error.")
        })?;

    Ok(HttpResponse::Ok().json(sump_events))
}

fn sump_events(db: Data<dyn DbPool>) -> Result<Vec<SumpEvent>, Error> {
    let mut conn = db.get_conn().expect("Could not get a db connection.");
    let sump_events: Vec<SumpEvent> = SumpEvent::all()
        .limit(100)
        .load::<SumpEvent>(&mut conn)
        .map_err(|e| anyhow!(e))?;

    Ok(sump_events)
}
