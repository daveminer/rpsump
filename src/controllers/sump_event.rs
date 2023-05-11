use crate::auth::authenticated_user::AuthenticatedUser;
use crate::database::{self, DbPool};
use crate::models::sump_event::SumpEvent;
use actix_web::error;
use actix_web::{get, web, web::Data, HttpResponse, Responder, Result};
use diesel::RunQueryDsl;

#[get("/sump_event")]
async fn sump_event(
    _req_body: String,
    db: Data<DbPool>,
    // TODO: check if needed
    _user: AuthenticatedUser,
) -> Result<impl Responder> {
    let _events = web::block(move || {
        let mut conn = database::conn(db);
        SumpEvent::all().load::<SumpEvent>(&mut conn)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;

    //Ok(HttpResponse::Ok().body(format!("{:?}", events)))
    Ok(HttpResponse::Ok().body(""))
}

// #[get("/pump_runs")]
// async fn pump_runs() -> Result<impl Responder> {
//     Ok(HttpResponse::Ok().body("Pump runs"))
// }
