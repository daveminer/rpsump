use actix_web::{post, web::Data, HttpResponse, Result};
use serde_json::json;

use crate::auth::authenticated_user::AuthenticatedUser;
use crate::controllers::auth::helpers::error_response;
use crate::repository::Repo;
use crate::util::ApiResponse;

#[post("/stop")]
#[tracing::instrument(skip(repo, _user))]
pub async fn stop(repo: Data<Repo>, _user: AuthenticatedUser) -> Result<HttpResponse> {
    match repo.request_garden_stop().await {
        Ok(ids) if ids.is_empty() => Ok(ApiResponse::bad_request(
            "No garden event is currently running or queued".into(),
        )),
        Ok(ids) => Ok(HttpResponse::Ok().json(json!({
            "cancelled_event_id": ids.first(),
            "cancelled_event_ids": ids,
        }))),
        Err(e) => Ok(error_response(e, "Could not stop garden event")),
    }
}
