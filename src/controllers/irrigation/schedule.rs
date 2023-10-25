use actix_web::{delete, error, get, patch, post, web, web::Data, HttpResponse, Responder, Result};
use chrono::NaiveTime;
use diesel::result::Error::NotFound;
use diesel::RunQueryDsl;

use crate::auth::authenticated_user::AuthenticatedUser;
use crate::controllers::{spawn_blocking_with_tracing, ApiResponse};
use crate::database::DbPool;
use crate::models::irrigation_schedule::{DayOfWeek, IrrigationSchedule};

#[derive(Debug, serde::Deserialize)]
pub struct IrrigationScheduleParams {
    pub days_of_week: Option<Vec<DayOfWeek>>,
    pub hoses: Option<Vec<i32>>,
    pub name: Option<String>,
    pub duration: Option<i32>,
    pub start_time: Option<NaiveTime>,
}

#[get("/schedule")]
#[tracing::instrument(skip(_req_body, db, _user))]
pub async fn irrigation_schedules(
    _req_body: String,
    db: Data<DbPool>,
    _user: AuthenticatedUser,
) -> Result<impl Responder> {
    let schedules = spawn_blocking_with_tracing(move || {
        let mut conn = db.get().expect("Could not get a db connection.");
        IrrigationSchedule::all().get_results::<IrrigationSchedule>(&mut conn)
    })
    .await
    .map_err(|e| {
        tracing::error!(
            target = module_path!(),
            error = e.to_string(),
            "Error while spawning a blocking task",
        );
        error::ErrorInternalServerError("Internal server error.")
    })?
    .map_err(|e| {
        tracing::error!(
            target = module_path!(),
            error = e.to_string(),
            "Error while getting irrigation schedules"
        );
        error::ErrorInternalServerError("Internal server error.")
    })?;

    Ok(HttpResponse::Ok().json(schedules))
}

#[get("/schedule/{id}")]
#[tracing::instrument(skip(db, _user))]
pub async fn irrigation_schedule(
    path: web::Path<i32>,
    db: Data<DbPool>,
    _user: AuthenticatedUser,
) -> Result<impl Responder> {
    let id = path.into_inner();
    let irrigation_schedule = spawn_blocking_with_tracing(move || {
        let mut conn = db.get().expect("Could not get a db connection.");
        return IrrigationSchedule::by_user_id(id).first::<IrrigationSchedule>(&mut conn);
    })
    .await
    .map_err(|e| {
        tracing::error!(
            target = module_path!(),
            error = e.to_string(),
            "Error while spawning a blocking task"
        );
        error::ErrorInternalServerError("Internal server error.")
    })?
    .map_err(|e| {
        if e == NotFound {
            return error::ErrorNotFound(serde_json::json!({
                "message": "Irrigation schedule not found."
            }));
        };

        tracing::error!(
            target = module_path!(),
            error = e.to_string(),
            "Error while getting irrigation schedules"
        );

        error::ErrorInternalServerError("Internal server error.")
    })?;

    Ok(HttpResponse::Ok().json(irrigation_schedule))
}

#[delete("/schedule/{id}")]
#[tracing::instrument(skip(db, _user))]
pub async fn delete_irrigation_schedule(
    path: web::Path<i32>,
    db: Data<DbPool>,
    _user: AuthenticatedUser,
) -> Result<impl Responder> {
    let id = path.into_inner();

    let id = match IrrigationSchedule::delete(id, db).await {
        Ok(id) => id,
        Err(e) => {
            // TODO: Handle not found
            return Ok(ApiResponse::bad_request(e.to_string()));
        }
    };

    Ok(HttpResponse::Ok().json(id))
}

#[patch("/schedule/{id}")]
#[tracing::instrument(skip(req_body, db, _user))]
pub async fn edit_irrigation_schedule(
    path: web::Path<i32>,
    req_body: web::Json<IrrigationScheduleParams>,
    db: Data<DbPool>,
    _user: AuthenticatedUser,
) -> Result<impl Responder> {
    let id = path.into_inner();

    // Create an irrigation schedule entry.
    let updated_irrigation_schedule = IrrigationSchedule::edit(
        id,
        req_body.hoses.clone(),
        req_body.name.clone(),
        req_body.start_time.clone(),
        req_body.days_of_week.clone(),
        db.clone(),
    )
    .await;

    return match updated_irrigation_schedule {
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
    };
}

#[post("/schedule")]
#[tracing::instrument(skip(req_body, db, _user))]
pub async fn new_irrigation_schedule(
    req_body: web::Json<IrrigationScheduleParams>,
    db: Data<DbPool>,
    _user: AuthenticatedUser,
) -> Result<impl Responder> {
    let new_irrigation_schedule = IrrigationSchedule::create(
        req_body.hoses.clone().unwrap(),
        req_body.name.clone().unwrap(),
        req_body.start_time.unwrap(),
        req_body.duration.unwrap(),
        req_body.days_of_week.clone().unwrap(),
        db.clone(),
    )
    .await;

    let response = match new_irrigation_schedule {
        Ok(schedule) => schedule,
        Err(e) => {
            // TODO: check bad request or ISE
            return Ok(ApiResponse::bad_request(e.to_string()));
        }
    };

    Ok(HttpResponse::Ok().json(response))
}
