use actix_web::{error, get, web::Data, HttpResponse, Responder, Result};
use anyhow::{anyhow, Error};
use diesel::{QueryDsl, RunQueryDsl};

use crate::auth::authenticated_user::AuthenticatedUser;
use crate::controllers::ApiResponse;
use crate::database::{self, DbPool};
use crate::models::sump_event::SumpEvent;

#[get("/sump_event")]
#[tracing::instrument(skip(_req_body, db, _user))]
pub async fn sump_event(
    _req_body: String,
    db: Data<DbPool>,
    _user: AuthenticatedUser,
) -> Result<impl Responder> {
    let events = web::block(move || {
        let mut conn = database::conn(db);
        SumpEvent::all().limit(100).load::<SumpEvent>(&mut conn)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;

    Ok(ApiResponse::ok(format!("{:?}", events)))
}
