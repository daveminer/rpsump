use actix_web::{error, get, web::Data, HttpResponse, Responder, Result};
use actix_web::{web, HttpRequest};
use anyhow::{anyhow, Error};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use serde::Deserialize;

use crate::auth::authenticated_user::AuthenticatedUser;
use crate::controllers::spawn_blocking_with_tracing;
use crate::database::{self, DbPool};
use crate::models::irrigation_event::IrrigationEvent;
use crate::schema::irrigation_event::status;

#[derive(Debug, Deserialize)]
pub struct Params {
    status: Option<String>,
}

#[get("/event")]
#[tracing::instrument(skip(req, db, _user))]
pub async fn irrigation_event(
    req: HttpRequest,
    db: Data<DbPool>,
    _user: AuthenticatedUser,
) -> Result<impl Responder> {
    let filter = match web::Query::<Params>::from_query(req.query_string()) {
        Ok(filter) => filter,
        Err(_e) => {
            return Ok(HttpResponse::BadRequest().body("invalid filter"));
        }
    };

    let irrigation_events =
        spawn_blocking_with_tracing(move || irrigation_events(db, filter.status.clone()))
            .await
            .map_err(|e| {
                tracing::error!("Error while spawning a blocking task: {:?}", e);
                error::ErrorInternalServerError("Internal server error.")
            })?
            .map_err(|e| {
                tracing::error!("Error while getting sump events: {:?}", e);
                error::ErrorInternalServerError("Internal server error.")
            })?;

    Ok(HttpResponse::Ok().json(irrigation_events))
}

fn irrigation_events(
    db: Data<DbPool>,
    filter_status: Option<String>,
) -> Result<Vec<IrrigationEvent>, Error> {
    let mut conn = database::conn(db)?;
    let mut query = IrrigationEvent::all().limit(100);
    if let Some(filter_status) = filter_status {
        query = query.filter(status.eq(filter_status))
    }

    let events = query
        .load::<IrrigationEvent>(&mut conn)
        .map_err(|e| anyhow!(e))?;

    Ok(events)
}
