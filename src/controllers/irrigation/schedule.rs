use actix_web::{delete, error, get, patch, post, web, web::Data, HttpResponse, Responder, Result};
use anyhow::{anyhow, Error};
use chrono::NaiveDateTime;
use diesel::{QueryDsl, RunQueryDsl};

use crate::auth::authenticated_user::AuthenticatedUser;
use crate::controllers::{spawn_blocking_with_tracing, ApiResponse};
use crate::database::{self, DbPool};
use crate::models::irrigation_schedule::{DayOfWeek, IrrigationSchedule};

#[derive(Debug, serde::Deserialize)]
pub struct IrrigationScheduleParams {
    pub days_of_week: Vec<DayOfWeek>,
    pub hoses: Vec<i32>,
    pub name: String,
    pub start_time: NaiveDateTime,
}

#[get("/schedule/{id}")]
#[tracing::instrument(skip(_req_body, db, _user))]
pub async fn irrigation_schedule(
    _req_body: String,
    db: Data<DbPool>,
    _user: AuthenticatedUser,
) -> Result<impl Responder> {
    let irrigation_schedule = spawn_blocking_with_tracing(move || fetch_irrigation_schedule(db))
        .await
        .map_err(|e| {
            tracing::error!("Error while spawning a blocking task: {:?}", e);
            error::ErrorInternalServerError("Internal server error.")
        })?
        .map_err(|e| {
            tracing::error!("Error while getting sump events: {:?}", e);
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
            return Ok(ApiResponse::bad_request(e.to_string()));
        }
    };

    Ok(HttpResponse::Ok().json(id))
}

#[patch("/irrigation_schedule/{id}")]
//#[tracing::instrument(skip(_req_body, db, _user))]
pub async fn edit_irrigation_schedule(
    path: web::Path<i32>,
    req_body: web::Json<IrrigationScheduleParams>,
    db: Data<DbPool>,
    _user: AuthenticatedUser,
) -> Result<impl Responder> {
    let id = path.into_inner();

    // Create an irrigation schedule entry.
    let updated_irrigation_schedule = match IrrigationSchedule::edit(
        id,
        "req_body_name".into(),
        req_body.start_time.clone(),
        req_body.days_of_week.clone(),
        db.clone(),
    )
    .await
    {
        Ok(schedule) => schedule,
        Err(e) => {
            return Ok(ApiResponse::bad_request(e.to_string()));
        }
    };

    Ok(HttpResponse::Ok().json(updated_irrigation_schedule))
}

#[post("/schedule")]
#[tracing::instrument(skip(req_body, db, _user))]
pub async fn new_irrigation_schedule(
    req_body: web::Json<IrrigationScheduleParams>,
    db: Data<DbPool>,
    _user: AuthenticatedUser,
) -> Result<impl Responder> {
    let new_irrigation_schedule = IrrigationSchedule::create(
        req_body.hoses.clone(),
        req_body.name.clone(),
        req_body.start_time,
        req_body.days_of_week.clone(),
        db.clone(),
    )
    .await;

    let response = match new_irrigation_schedule {
        Ok(schedule) => schedule,
        Err(e) => {
            return Ok(ApiResponse::bad_request(e.to_string()));
        }
    };

    Ok(HttpResponse::Ok().json(response))
}

fn fetch_irrigation_schedule(db: Data<DbPool>) -> Result<Vec<IrrigationSchedule>, Error> {
    let mut conn = database::conn(db)?;
    let irrigation_events: Vec<IrrigationSchedule> = IrrigationSchedule::all()
        .limit(100)
        .load::<IrrigationSchedule>(&mut conn)
        .map_err(|e| anyhow!(e))?;

    Ok(irrigation_events)
}
