use actix_web::{web, error};
use actix_web::web::Data;
use anyhow::{anyhow, Error};
use chrono::{NaiveDateTime, NaiveTime};
use diesel::prelude::*;
use diesel::result::Error::NotFound;
use diesel::sqlite::Sqlite;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::database;
use crate::database::DbPool;
use crate::schema::irrigation_schedule;
use crate::schema::irrigation_schedule::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DayOfWeek {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

#[derive(AsChangeset, Clone, Debug, PartialEq, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = irrigation_schedule)]
pub struct IrrigationSchedule {
    pub id: i32,
    pub active: bool,
    pub name: String,
    pub start_time: NaiveTime,
    pub days_of_week: String,
    pub hoses: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

type BoxedQuery<'a> = irrigation_schedule::BoxedQuery<'a, Sqlite, irrigation_schedule::SqlType>;

impl fmt::Display for DayOfWeek {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DayOfWeek::Monday => write!(f, "Monday"),
            DayOfWeek::Tuesday => write!(f, "Tuesday"),
            DayOfWeek::Wednesday => write!(f, "Wednesday"),
            DayOfWeek::Thursday => write!(f, "Thursday"),
            DayOfWeek::Friday => write!(f, "Friday"),
            DayOfWeek::Saturday => write!(f, "Saturday"),
            DayOfWeek::Sunday => write!(f, "Sunday"),
        }
    }
}


impl IrrigationSchedule {
    // Composable queries
    pub fn active() -> BoxedQuery<'static> {
        IrrigationSchedule::all().filter(irrigation_schedule::active.eq(true))
    }

    pub fn all() -> BoxedQuery<'static> {
        irrigation_schedule::table.limit(100).into_boxed()
    }

    pub fn by_id(user_id: i32) -> BoxedQuery<'static> {
        irrigation_schedule::table
            .filter(irrigation_schedule::id.eq(user_id))
            .into_boxed()
    }

    pub async fn create(
        schedule_hoses: Vec<i32>,
        schedule_name: String,
        schedule_start_time: NaiveTime,
        schedule_days_of_week: Vec<DayOfWeek>,
        db: Data<DbPool>,
    ) -> Result<IrrigationSchedule, Error> {
        // TODO: block with tracing
        let new_schedule = web::block(move || {
            let mut conn = db.get().expect("Could not get a db connection.");

            let hose_str = schedule_hoses
                .iter()
                .map(|hose| hose.to_string())
                .collect::<Vec<String>>()
                .join(",");

            let day_of_week_str = schedule_days_of_week
                .iter()
                .map(|day| day.to_string())
                .collect::<Vec<String>>()
                .join(",");

            return diesel::insert_into(irrigation_schedule::table)
                .values((
                    active.eq(true),
                    hoses.eq(hose_str),
                    name.eq(schedule_name),
                    start_time.eq(schedule_start_time),
                    days_of_week.eq(day_of_week_str),
                ))
                .get_result::<IrrigationSchedule>(&mut conn);
        })
        .await
        .map_err(|e| anyhow!("Error: {e}"))?
        .map_err(|e| anyhow!("Internal server error when creating irrigation schedule: {e}"))?;


        Ok(new_schedule)
    }

    pub async fn delete(schedule_id: i32, db: Data<DbPool>) -> Result<IrrigationSchedule, Error> {
        let result = web::block(move || {
            let mut conn = db.get().expect("Could not get a db connection.");

            diesel::delete(irrigation_schedule::table.find(schedule_id))
                .get_result::<IrrigationSchedule>(&mut conn)
                .map_err(|e| anyhow!("Error deleting irrigation schedules: {}", e))
        })
        .await??;

        Ok(result)
    }

    pub async fn edit(
        schedule_id: i32,
        schedule_hoses: Option<Vec<i32>>,
        schedule_name: Option<String>,
        schedule_start_time: Option<NaiveTime>,
        schedule_days_of_week: Option<Vec<DayOfWeek>>,
        db: Data<DbPool>,
    ) -> Result<Option<IrrigationSchedule>, Error> {
        let updated_schedule = web::block(move || {
            let mut conn = db.get().expect("Could not get a db connection.");

            let existing_schedule = irrigation_schedule::table
                .find(schedule_id)
                .first::<IrrigationSchedule>(&mut conn)
                .optional()
                .map_err(|e| anyhow!("Error checking for existing irrigation schedule: {}", e))?;

            println!("EXISTING: {:?}", existing_schedule);
            if existing_schedule.is_none() {
                println!("HERE");
                return Ok(None);
            }

            let mut updated_schedule = existing_schedule.unwrap().clone();

            println!("UPDATED");
            if let Some(schedule_hoses) = schedule_hoses {
                updated_schedule.hoses = schedule_hoses
                    .iter()
                    .map(|hose| hose.to_string())
                    .collect::<Vec<String>>()
                    .join(",");
            }

            if let Some(schedule_name) = schedule_name {
                updated_schedule.name = schedule_name;
            }

            if let Some(schedule_start_time) = schedule_start_time {
                updated_schedule.start_time = schedule_start_time;
            }

            if let Some(schedule_days_of_week) = schedule_days_of_week {
                updated_schedule.days_of_week = schedule_days_of_week.iter().map(|day| {
                    day.to_string()
                }).collect::<Vec<String>>().join(",");
            }

            let result = diesel::update(irrigation_schedule::table.find(schedule_id))
                        .set(updated_schedule)
                        .get_result::<IrrigationSchedule>(&mut conn)
                        .map_err(|e| anyhow!("Error updating irrigation event: {}", e));

            match result {
                Ok(schedule) => Ok(Some(schedule)),
                Err(e) => Err(anyhow!("Error updating irrigation event: {}", e))
            }
        })
        .await
        .map_err(|e| {
            println!("EEeeeEESS: {}", e);
            tracing::error!("Error while spawning a blocking task: {:?}", e);
            error::ErrorInternalServerError("Internal server error.")
        })
        .map_err(|e| {
            println!("EEEESS: {}", e);

            tracing::error!("Error while getting irrigation schedules: {:?}", e);
            error::ErrorInternalServerError("Internal server error.")
        });

        println!("UPDSSS: {:?}", updated_schedule);
        match updated_schedule {
            Ok(schedule) => schedule,
            Err(e) => Err(anyhow!("Error while updating irrigation schedule: {}", e))
        }
    }

    pub fn fetch_irrigation_schedule(
        db: Data<DbPool>,
        schedule_id: Option<i32>,
    ) -> Result<Vec<IrrigationSchedule>, Error> {
        let mut conn = database::conn(db)?;
        let query: BoxedQuery<'static> = if schedule_id.is_some() {
            irrigation_schedule::table
                .find(schedule_id.unwrap())
                .into_boxed()
        } else {
            irrigation_schedule::table.limit(100).into_boxed()
        };

        let irrigation_events: Vec<IrrigationSchedule> = query
            .load::<IrrigationSchedule>(&mut conn)
            .map_err(|e| anyhow!(e))?;

        Ok(irrigation_events)
    }
}
