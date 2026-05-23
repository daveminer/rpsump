use actix_web::{get, web::Data, HttpResponse, Result};
use chrono::{NaiveDateTime, Utc};
use serde::Serialize;
use tokio::sync::Mutex;

use crate::auth::authenticated_user::AuthenticatedUser;
use crate::controllers::auth::helpers::error_response;
use crate::hydro::garden::schedule::next_scheduled_run;
use crate::hydro::Hydro;
use crate::repository::models::garden_event::GardenEvent;
use crate::repository::Repo;

#[derive(Debug, Serialize)]
struct StatusResponse {
    is_on: bool,
    current_event: Option<GardenEvent>,
    next_run_at: Option<NaiveDateTime>,
    next_run_schedule_id: Option<i32>,
}

#[get("/status")]
#[tracing::instrument(skip(repo, hydro, _user))]
pub async fn status(
    repo: Data<Repo>,
    hydro: Data<Mutex<Hydro>>,
    _user: AuthenticatedUser,
) -> Result<HttpResponse> {
    let is_on = {
        let hydro = hydro.lock().await;
        hydro.garden.is_on().await
    };

    let current_event = match repo.current_garden_event().await {
        Ok(e) => e,
        Err(e) => return Ok(error_response(e, "Could not get current garden event")),
    };

    let schedules = match repo.garden_schedules().await {
        Ok(s) => s,
        Err(e) => return Ok(error_response(e, "Could not get garden schedules")),
    };

    let next = next_scheduled_run(&schedules, Utc::now().naive_utc());

    Ok(HttpResponse::Ok().json(StatusResponse {
        is_on,
        current_event,
        next_run_at: next.map(|(_, t)| t),
        next_run_schedule_id: next.map(|(id, _)| id),
    }))
}
