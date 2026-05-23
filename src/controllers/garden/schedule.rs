use actix_web::{delete, get, patch, post, web, web::Data, HttpResponse, Result};

use crate::auth::authenticated_user::AuthenticatedUser;
use crate::controllers::auth::helpers::error_response;
use crate::repository::models::garden_schedule::{
    CreateGardenScheduleParams, UpdateGardenScheduleParams,
};
use crate::repository::Repo;
use crate::util::ApiResponse;

#[get("/schedule")]
#[tracing::instrument(skip(repo, _user))]
pub async fn list_schedules(
    repo: Data<Repo>,
    _user: AuthenticatedUser,
) -> Result<HttpResponse> {
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
#[tracing::instrument(skip(req_body, repo, _user))]
pub async fn create_schedule(
    req_body: web::Json<CreateGardenScheduleParams>,
    repo: Data<Repo>,
    _user: AuthenticatedUser,
) -> Result<HttpResponse> {
    let params = req_body.into_inner();
    if params.start_times.is_empty() {
        return Ok(ApiResponse::bad_request(
            "at least one start_time is required".into(),
        ));
    }
    if params.days_of_week.is_empty() {
        return Ok(ApiResponse::bad_request(
            "at least one day_of_week is required".into(),
        ));
    }
    if params.duration_secs <= 0 {
        return Ok(ApiResponse::bad_request(
            "duration_secs must be positive".into(),
        ));
    }

    match repo.create_garden_schedule(params).await {
        Ok(s) => Ok(HttpResponse::Ok().json(s)),
        Err(e) => Ok(ApiResponse::bad_request(e.to_string())),
    }
}

#[patch("/schedule/{id}")]
#[tracing::instrument(skip(req_body, repo, _user))]
pub async fn update_schedule(
    path: web::Path<i32>,
    req_body: web::Json<UpdateGardenScheduleParams>,
    repo: Data<Repo>,
    _user: AuthenticatedUser,
) -> Result<HttpResponse> {
    let id = path.into_inner();
    let params = req_body.into_inner();
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
