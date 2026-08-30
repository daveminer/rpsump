use actix_web::{post, web, web::Data, HttpResponse, Result};
use serde::Deserialize;
use serde_json::json;
use tokio::sync::Mutex;

use crate::auth::authenticated_user::AuthenticatedUser;
use crate::controllers::auth::helpers::error_response;
use crate::hydro::Hydro;
use crate::repository::Repo;
use crate::util::ApiResponse;

#[derive(Debug, Deserialize)]
pub struct RunParams {
    pub duration_secs: i32,
}

#[post("/run")]
#[tracing::instrument(skip(repo, hydro, _user))]
pub async fn run_now(
    req_body: web::Json<RunParams>,
    repo: Data<Repo>,
    hydro: Data<Mutex<Hydro>>,
    _user: AuthenticatedUser,
) -> Result<HttpResponse> {
    let RunParams { duration_secs } = req_body.into_inner();

    if duration_secs <= 0 {
        return Ok(ApiResponse::bad_request(
            "duration_secs must be positive".into(),
        ));
    }

    let max_seconds_runtime = {
        let hydro = hydro.lock().await;
        hydro.garden.max_seconds_runtime
    };

    if duration_secs > max_seconds_runtime as i32 {
        return Ok(ApiResponse::bad_request(format!(
            "duration_secs must be at most {}",
            max_seconds_runtime
        )));
    }

    let event = match repo.create_manual_garden_event(duration_secs).await {
        Ok(Some(event)) => event,
        Ok(None) => {
            return Ok(HttpResponse::Conflict().json(json!({
                "message": "A garden event is already queued or running"
            })))
        }
        Err(e) => return Ok(error_response(e, "Could not queue manual garden event")),
    };

    // The scheduler sleeps between ticks; wake it so the run starts now rather
    // than at the top of the next tick.
    {
        let hydro = hydro.lock().await;
        hydro.garden.wake_scheduler();
    }

    Ok(HttpResponse::Accepted().json(event))
}
