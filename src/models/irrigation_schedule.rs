use actix_web::web;
use actix_web::web::Data;
use anyhow::{anyhow, Error};
use chrono::NaiveDateTime;
use diesel::backend::Backend;
use diesel::dsl::*;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

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
    pub name: String,
    pub start_time: NaiveDateTime,
    pub days_of_week: String,
    pub hoses: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

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
    pub async fn create(
        schedule_hoses: Vec<i32>,
        schedule_name: String,
        schedule_start_time: NaiveDateTime,
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
                    hoses.eq(hose_str),
                    name.eq(schedule_name),
                    start_time.eq(schedule_start_time),
                    days_of_week.eq(day_of_week_str),
                ))
                .get_result::<IrrigationSchedule>(&mut conn);
        })
        .await?
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
        .await?
        .map_err(|e| anyhow!("Internal server error when deleting irrigation schedule: {e}"))?;

        Ok(result)
    }

    pub async fn edit(
        schedule_id: i32,
        schedule_name: String,
        schedule_start_time: NaiveDateTime,
        schedule_days_of_week: Vec<DayOfWeek>,
        db: Data<DbPool>,
    ) -> Result<(), Error> {
        web::block(move || {
            let mut conn = db.get().expect("Could not get a db connection.");

            let existing_schedule = irrigation_schedule::table
                .find(schedule_id)
                .first::<IrrigationSchedule>(&mut conn)
                .optional()
                .map_err(|e| anyhow!("Error checking for existing irrigation schedule: {}", e))?;

            if let Some(schedule) = existing_schedule {
                let days_of_week_string = schedule_days_of_week
                    .iter()
                    .map(|day| match day {
                        DayOfWeek::Monday => "Monday",
                        DayOfWeek::Tuesday => "Tuesday",
                        DayOfWeek::Wednesday => "Wednesday",
                        DayOfWeek::Thursday => "Thursday",
                        DayOfWeek::Friday => "Friday",
                        DayOfWeek::Saturday => "Saturday",
                        DayOfWeek::Sunday => "Sunday",
                    })
                    .collect::<Vec<&str>>()
                    .join(",");

                let updated_schedule = IrrigationSchedule {
                    name: schedule_name,
                    start_time: schedule_start_time,
                    days_of_week: days_of_week_string,
                    ..schedule
                };

                diesel::update(irrigation_schedule::table.find(schedule_id))
                    .set(updated_schedule)
                    .execute(&mut conn)
                    .map_err(|e| anyhow!("Error updating irrigation event: {}", e))?;
            } else {
                return Err(anyhow!("No irrigation event found with ID {}", schedule_id));
            }

            Ok(())
        })
        .await?
        .map_err(|e| anyhow!("Internal server error when editing irrigation schedule: {e}"))?;

        Ok(())
    }

    pub fn all<DB>() -> Select<irrigation_schedule::table, AsSelect<IrrigationSchedule, DB>>
    where
        DB: Backend,
    {
        irrigation_schedule::table.select(IrrigationSchedule::as_select())
    }
}
