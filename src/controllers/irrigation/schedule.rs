use actix_web::{delete, error, get, patch, post, web, web::Data, HttpResponse, Responder, Result};
use anyhow::{anyhow, Error};
use chrono::NaiveDateTime;
use diesel::{QueryDsl, RunQueryDsl};

use crate::auth::authenticated_user::AuthenticatedUser;
use crate::controllers::{spawn_blocking_with_tracing, ApiResponse};
use crate::database::{self, DbPool};
use crate::models::irrigation_schedule::{DayOfWeek, IrrigationSchedule};

#[get("/irrigation_schedule/{id}")]
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

#[delete("/irrigation_schedule/{id}")]
#[tracing::instrument(skip(req_body, db, _user))]
pub async fn delete_irrigation_schedule(
    path: web::Path<i32>,
    req_body: String,
    db: Data<DbPool>,
    _user: AuthenticatedUser,
) -> Result<impl Responder> {
    println!("req_body: {:?}", req_body);
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
    req_body: String,
    db: Data<DbPool>,
    _user: AuthenticatedUser,
) -> Result<impl Responder> {
    let id = path.into_inner();

    let start_time: NaiveDateTime =
        NaiveDateTime::parse_from_str("2021-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();

    let days_of_week: Vec<DayOfWeek> = vec![DayOfWeek::Monday, DayOfWeek::Tuesday];
    // Create an irrigation schedule entry.
    let updated_irrigation_schedule = match IrrigationSchedule::edit(
        id,
        Some("req_body_name".into()),
        Some(start_time),
        Some(days_of_week),
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

#[post("/irrigation_schedule")]
#[tracing::instrument(skip(req_body, db, _user))]
pub async fn new_irrigation_schedule(
    req_body: String,
    db: Data<DbPool>,
    _user: AuthenticatedUser,
) -> Result<impl Responder> {
    println!("req_body: {:?}", req_body);

    let start_time: NaiveDateTime =
        NaiveDateTime::parse_from_str("2021-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();

    let days_of_week: Vec<DayOfWeek> = vec![DayOfWeek::Monday, DayOfWeek::Tuesday];
    // Create an irrigation schedule entry.
    let new_irrigation_schedule = match IrrigationSchedule::create(
        "req_body_name".into(),
        start_time,
        days_of_week,
        db.clone(),
    )
    .await
    {
        Ok(schedule) => schedule,
        Err(e) => {
            return Ok(ApiResponse::bad_request(e.to_string()));
        }
    };

    Ok(HttpResponse::Ok().json(new_irrigation_schedule))
}

fn fetch_irrigation_schedule(db: Data<DbPool>) -> Result<Vec<IrrigationSchedule>, Error> {
    let mut conn = database::conn(db)?;
    let irrigation_events: Vec<IrrigationSchedule> = IrrigationSchedule::all()
        .limit(100)
        .load::<IrrigationSchedule>(&mut conn)
        .map_err(|e| anyhow!(e))?;

    Ok(irrigation_events)
}
