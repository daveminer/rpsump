use actix_web::{
    get,
    web::{Data, Query},
    HttpResponse, Result,
};
use serde::Deserialize;
use std::str::FromStr;

use crate::auth::authenticated_user::AuthenticatedUser;
use crate::controllers::auth::helpers::error_response;
use crate::repository::models::garden_event::{
    GardenEventFilter, GardenEventSource, GardenEventStatus,
};
use crate::repository::Repo;
use crate::util::{parse_datetime_param, ApiResponse};

#[derive(Debug, Deserialize)]
pub struct EventQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub source: Option<String>,
    pub status: Option<String>,
    /// Inclusive bounds on `scheduled_for`, which every event carries.
    pub from: Option<String>,
    pub to: Option<String>,
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

    let status = match query.status.as_deref() {
        None => None,
        Some(s) => match GardenEventStatus::from_str(s) {
            Ok(status) => Some(status),
            Err(_) => {
                return Ok(ApiResponse::bad_request(
                    "status must be one of 'queued', 'in_progress', 'completed', 'cancelled', 'skipped'"
                        .into(),
                ))
            }
        },
    };

    let from = match query.from.as_deref().map(parse_datetime_param) {
        Some(None) => return Ok(bad_timestamp("from")),
        Some(Some(dt)) => Some(dt),
        None => None,
    };

    let to = match query.to.as_deref().map(parse_datetime_param) {
        Some(None) => return Ok(bad_timestamp("to")),
        Some(Some(dt)) => Some(dt),
        None => None,
    };

    if let (Some(from), Some(to)) = (from, to) {
        if from > to {
            return Ok(ApiResponse::bad_request(
                "from must not be later than to".into(),
            ));
        }
    }

    let filter = GardenEventFilter {
        limit: Some(limit),
        offset: Some(offset),
        source,
        status,
        from,
        to,
    };

    match repo.garden_events(filter).await {
        Ok(events) => Ok(HttpResponse::Ok().json(events)),
        Err(e) => Ok(error_response(e, "Could not get garden events")),
    }
}

fn bad_timestamp(field: &str) -> HttpResponse {
    ApiResponse::bad_request(format!(
        "{} must be an ISO-8601 timestamp, e.g. 2026-08-29T18:30:00Z",
        field
    ))
}
