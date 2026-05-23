use actix_web::{
    get,
    web::{Data, Query},
    HttpResponse, Result,
};
use serde::Deserialize;
use std::str::FromStr;

use crate::auth::authenticated_user::AuthenticatedUser;
use crate::controllers::auth::helpers::error_response;
use crate::repository::models::garden_event::GardenEventSource;
use crate::repository::Repo;
use crate::util::ApiResponse;

#[derive(Debug, Deserialize)]
pub struct EventQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub source: Option<String>,
}

#[get("/event")]
#[tracing::instrument(skip(repo, _user))]
pub async fn list_events(
    query: Query<EventQuery>,
    repo: Data<Repo>,
    _user: AuthenticatedUser,
) -> Result<HttpResponse> {
    let limit = query.limit.unwrap_or(100).clamp(1, 500);
    let offset = query.offset.unwrap_or(0).max(0);

    let source = match query.source.as_deref() {
        None => None,
        Some(s) => match GardenEventSource::from_str(s) {
            Ok(src) => Some(src),
            Err(_) => {
                return Ok(ApiResponse::bad_request(
                    "source must be 'scheduled' or 'manual'".into(),
                ))
            }
        },
    };

    match repo.garden_events(limit, offset, source).await {
        Ok(events) => Ok(HttpResponse::Ok().json(events)),
        Err(e) => Ok(error_response(e, "Could not get garden events")),
    }
}
