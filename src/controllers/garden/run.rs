use actix_web::{post, web, web::Data, HttpResponse, Result};
use serde::Deserialize;

use crate::auth::authenticated_user::AuthenticatedUser;
use crate::controllers::auth::helpers::error_response;
use crate::repository::Repo;
use crate::util::ApiResponse;

#[derive(Debug, Deserialize)]
pub struct RunParams {
    pub duration_secs: i32,
}

#[post("/run")]
#[tracing::instrument(skip(repo, _user))]
pub async fn run_now(
    req_body: web::Json<RunParams>,
    repo: Data<Repo>,
    _user: AuthenticatedUser,
) -> Result<HttpResponse> {
    let RunParams { duration_secs } = req_body.into_inner();

    if duration_secs <= 0 {
        return Ok(ApiResponse::bad_request(
            "duration_secs must be positive".into(),
        ));
    }

    match repo.create_manual_garden_event(duration_secs).await {
        Ok(event) => Ok(HttpResponse::Accepted().json(event)),
        Err(e) => Ok(error_response(e, "Could not queue manual garden event")),
    }
}
