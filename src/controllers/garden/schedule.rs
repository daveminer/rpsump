use actix_web::{delete, get, patch, post, web, web::Data, HttpResponse, Result};
use chrono::{NaiveTime, Weekday};

use crate::auth::authenticated_user::AuthenticatedUser;
use crate::config::Settings;
use crate::controllers::auth::helpers::error_response;
use crate::repository::models::garden_schedule::{
    CreateGardenScheduleParams, UpdateGardenScheduleParams,
};
use crate::repository::Repo;
use crate::util::ApiResponse;

/// Shared by create and update so a PATCH cannot slip past the rules a POST
/// has to satisfy. Every field is optional here; `None` means "not changing".
fn validation_error(
    name: Option<&str>,
    start_times: Option<&[NaiveTime]>,
    days_of_week: Option<&[Weekday]>,
    duration_secs: Option<i32>,
    max_seconds_runtime: u32,
) -> Option<HttpResponse> {
    if let Some(name) = name {
        if name.trim().is_empty() {
            return Some(ApiResponse::bad_request("name is required".into()));
        }
    }

    if let Some(start_times) = start_times {
        if start_times.is_empty() {
            return Some(ApiResponse::bad_request(
                "at least one start_time is required".into(),
            ));
        }
    }

    if let Some(days_of_week) = days_of_week {
        if days_of_week.is_empty() {
            return Some(ApiResponse::bad_request(
                "at least one day_of_week is required".into(),
            ));
        }
    }

    if let Some(duration_secs) = duration_secs {
        if duration_secs <= 0 {
            return Some(ApiResponse::bad_request(
                "duration_secs must be positive".into(),
            ));
        }
        // Runs are clamped to this at the solenoid; rejecting here keeps the
        // stored duration honest so reports don't over-count.
        if duration_secs > max_seconds_runtime as i32 {
            return Some(ApiResponse::bad_request(format!(
                "duration_secs must be at most {}",
                max_seconds_runtime
            )));
        }
    }

    None
}

#[get("/schedule")]
#[tracing::instrument(skip(repo, _user))]
pub async fn list_schedules(repo: Data<Repo>, _user: AuthenticatedUser) -> Result<HttpResponse> {
    match repo.garden_schedules().await {
        Ok(schedules) => Ok(HttpResponse::Ok().json(schedules)),
        Err(e) => Ok(error_response(e, "Could not get garden schedules")),
    }
}

#[get("/schedule/{id}")]
#[tracing::instrument(skip(repo, _user))]
pub async fn get_schedule(
    path: web::Path<i32>,
    repo: Data<Repo>,
    _user: AuthenticatedUser,
) -> Result<HttpResponse> {
    let id = path.into_inner();
    match repo.garden_schedule_by_id(id).await {
        Ok(Some(s)) => Ok(HttpResponse::Ok().json(s)),
        Ok(None) => Ok(ApiResponse::not_found()),
        Err(e) => Ok(error_response(e, "Could not get garden schedule")),
    }
}

#[post("/schedule")]
#[tracing::instrument(skip(req_body, repo, settings, _user))]
pub async fn create_schedule(
    req_body: web::Json<CreateGardenScheduleParams>,
    repo: Data<Repo>,
    settings: Data<Settings>,
    _user: AuthenticatedUser,
) -> Result<HttpResponse> {
    let params = req_body.into_inner();

    if let Some(response) = validation_error(
        Some(&params.name),
        Some(&params.start_times),
        Some(&params.days_of_week),
        Some(params.duration_secs),
        settings.hydro.garden.max_seconds_runtime,
    ) {
        return Ok(response);
    }

    match repo.create_garden_schedule(params).await {
        Ok(s) => Ok(HttpResponse::Ok().json(s)),
        Err(e) => Ok(ApiResponse::bad_request(e.to_string())),
    }
}

#[patch("/schedule/{id}")]
#[tracing::instrument(skip(req_body, repo, settings, _user))]
pub async fn update_schedule(
    path: web::Path<i32>,
    req_body: web::Json<UpdateGardenScheduleParams>,
    repo: Data<Repo>,
    settings: Data<Settings>,
    _user: AuthenticatedUser,
) -> Result<HttpResponse> {
    let id = path.into_inner();
    let params = req_body.into_inner();

    if let Some(response) = validation_error(
        params.name.as_deref(),
        params.start_times.as_deref(),
        params.days_of_week.as_deref(),
        params.duration_secs,
        settings.hydro.garden.max_seconds_runtime,
    ) {
        return Ok(response);
    }

    match repo.update_garden_schedule(id, params).await {
        Ok(Some(s)) => Ok(HttpResponse::Ok().json(s)),
        Ok(None) => Ok(ApiResponse::not_found()),
        Err(e) => Ok(error_response(e, "Could not update garden schedule")),
    }
}

#[delete("/schedule/{id}")]
#[tracing::instrument(skip(repo, _user))]
pub async fn delete_schedule(
    path: web::Path<i32>,
    repo: Data<Repo>,
    _user: AuthenticatedUser,
) -> Result<HttpResponse> {
    let id = path.into_inner();
    match repo.delete_garden_schedule(id).await {
        Ok(Some(_)) => Ok(HttpResponse::Ok().finish()),
        Ok(None) => Ok(ApiResponse::not_found()),
        Err(e) => Ok(ApiResponse::bad_request(e.to_string())),
    }
}
