use actix_web::{delete, get, patch, post, web, web::Data, HttpResponse, Result};
use chrono::NaiveTime;

use crate::auth::authenticated_user::AuthenticatedUser;
use crate::controllers::auth::helpers::error_response;
use crate::models::irrigation_schedule::{DayOfWeek, IrrigationSchedule};
use crate::repository::Repo;
use crate::util::ApiResponse;

#[derive(Debug, serde::Deserialize)]
pub struct IrrigationScheduleParams {
    pub days_of_week: Option<Vec<DayOfWeek>>,
    pub hoses: Option<Vec<i32>>,
    pub name: Option<String>,
    pub duration: Option<i32>,
    pub start_time: Option<NaiveTime>,
}

#[get("/schedule")]
#[tracing::instrument(skip(_req_body, repo, _user))]
pub async fn irrigation_schedules(
    _req_body: String,
    repo: Data<Repo>,
    _user: AuthenticatedUser,
) -> Result<HttpResponse> {
    let schedules = match repo.irrigation_schedules().await {
        Ok(schedules) => schedules,
        Err(e) => {
            return Ok(error_response(e, "Could not get irrigation schedules"));
        }
    };

    Ok(HttpResponse::Ok().json(schedules))
}

#[get("/schedule/{id}")]
#[tracing::instrument(skip(repo, _user))]
pub async fn irrigation_schedule(
    path: web::Path<i32>,
    repo: Data<Repo>,
    _user: AuthenticatedUser,
) -> Result<HttpResponse> {
    let id = path.into_inner();
    let irrigation_schedule = match repo.irrigation_schedule_by_id(id).await {
        Ok(irrigation_schedule) => irrigation_schedule,
        Err(e) => return Ok(ApiResponse::not_found()),
    };

    Ok(HttpResponse::Ok().json(irrigation_schedule))
}

#[delete("/schedule/{id}")]
#[tracing::instrument(skip(repo, _user))]
pub async fn delete_irrigation_schedule(
    path: web::Path<i32>,
    repo: Data<Repo>,
    _user: AuthenticatedUser,
) -> Result<HttpResponse> {
    let id = path.into_inner();

    let id = match repo.delete_irrigation_schedule(id).await {
        Ok(id) => id,
        Err(e) => {
            // TODO: Handle not found
            return Ok(ApiResponse::bad_request(e.to_string()));
        }
    };

    Ok(HttpResponse::Ok().json(id))
}

#[patch("/schedule/{id}")]
#[tracing::instrument(skip(req_body, repo, _user))]
pub async fn edit_irrigation_schedule(
    path: web::Path<i32>,
    req_body: web::Json<IrrigationScheduleParams>,
    repo: Data<Repo>,
    _user: AuthenticatedUser,
) -> Result<HttpResponse> {
    let id = path.into_inner();

    let params: IrrigationScheduleParams = req_body.into_inner();

    let irrigation_sched = match repo.update_irrigation_schedule(id, params).await {
        Ok(irrigation_sched) => irrigation_sched,
        Err(e) => {
            return Ok(ApiResponse::bad_request(e.to_string()));
        }
    };

    match irrigation_sched {
        Ok(None) => Ok(HttpResponse::NotFound().finish()),
        Ok(schedule) => Ok(HttpResponse::Ok().json(schedule)),
        Err(e) => {
            tracing::error!(
                target = module_path!(),
                error = e.to_string(),
                id = id,
                "Error while updating irrigation schedule"
            );
            Ok(HttpResponse::InternalServerError().into())
        }
    }
}

#[post("/schedule")]
#[tracing::instrument(skip(req_body, repo, _user))]
pub async fn new_irrigation_schedule(
    req_body: web::Json<IrrigationScheduleParams>,
    repo: Data<Repo>,
    _user: AuthenticatedUser,
) -> Result<HttpResponse> {
    let params: IrrigationScheduleParams = req_body.into_inner();

    let new_irrigation_schedule = repo.create_irrigation_schedule(params);

    let response = match new_irrigation_schedule {
        Ok(schedule) => schedule,
        Err(e) => {
            // TODO: check bad request or ISE
            return Ok(ApiResponse::bad_request(e.to_string()));
        }
    };

    Ok(HttpResponse::Ok().json(response))
}
