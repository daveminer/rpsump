use actix_web::{get, web::Data, HttpResponse, Result};
use chrono::{Duration, NaiveDateTime, Utc};
use serde::Serialize;
use tokio::sync::Mutex;

use crate::auth::authenticated_user::AuthenticatedUser;
use crate::controllers::auth::helpers::error_response;
use crate::controllers::garden::report::{rain_skips_since, watered_secs_since};
use crate::hydro::garden::schedule::next_scheduled_run;
use crate::hydro::Hydro;
use crate::repository::models::garden_event::{GardenEvent, GardenEventFilter};
use crate::repository::Repo;

/// The rolling windows the Irrigation screen puts above the run history.
#[derive(Debug, Default, Serialize)]
struct WateringTotals {
    last_24h_secs: i64,
    last_3d_secs: i64,
    last_7d_secs: i64,
}

#[derive(Debug, Serialize)]
struct StatusResponse {
    is_on: bool,
    current_event: Option<GardenEvent>,
    next_run_at: Option<NaiveDateTime>,
    next_run_schedule_id: Option<i32>,
    watering_totals: WateringTotals,
    rain_skips_48h: i64,
    /// Longest run the hardware config permits, so the client can bound its
    /// duration controls instead of guessing.
    max_runtime_secs: u32,
}

#[get("/status")]
#[tracing::instrument(skip(repo, hydro, _user))]
pub async fn status(
    repo: Data<Repo>,
    hydro: Data<Mutex<Hydro>>,
    _user: AuthenticatedUser,
) -> Result<HttpResponse> {
    let (is_on, max_runtime_secs) = {
        let hydro = hydro.lock().await;
        (
            hydro.garden.is_on().await,
            hydro.garden.max_seconds_runtime,
        )
    };

    let current_event = match repo.current_garden_event().await {
        Ok(e) => e,
        Err(e) => return Ok(error_response(e, "Could not get current garden event")),
    };

    let schedules = match repo.garden_schedules().await {
        Ok(s) => s,
        Err(e) => return Ok(error_response(e, "Could not get garden schedules")),
    };

    let now = Utc::now().naive_utc();
    let next = next_scheduled_run(&schedules, now);

    // One week covers every window this response reports.
    let recent = match repo
        .garden_events(GardenEventFilter::since(now - Duration::days(7)))
        .await
    {
        Ok(events) => events,
        Err(e) => return Ok(error_response(e, "Could not get garden events")),
    };

    let watering_totals = WateringTotals {
        last_24h_secs: watered_secs_since(&recent, now - Duration::hours(24), now),
        last_3d_secs: watered_secs_since(&recent, now - Duration::days(3), now),
        last_7d_secs: watered_secs_since(&recent, now - Duration::days(7), now),
    };

    Ok(HttpResponse::Ok().json(StatusResponse {
        is_on,
        current_event,
        next_run_at: next.map(|(_, t)| t),
        next_run_schedule_id: next.map(|(id, _)| id),
        watering_totals,
        rain_skips_48h: rain_skips_since(&recent, now - Duration::hours(48)),
        max_runtime_secs,
    }))
}
