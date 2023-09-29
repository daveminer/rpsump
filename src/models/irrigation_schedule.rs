use actix_web::web;
use anyhow::{anyhow, Error};
use chrono::NaiveDateTime;
use diesel::backend::Backend;
use diesel::dsl::*;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::database::DbPool;
use crate::schema::irrigation_schedule;
use crate::schema::irrigation_schedule::*;

pub enum DayOfWeek {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

#[derive(Clone, Debug, PartialEq, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = irrigation_schedule)]
pub struct IrrigationSchedule {
    pub id: i32,
    pub name: String,
    pub start_time: NaiveDateTime,
    pub days_of_week: Option<NaiveDateTime>,
    pub hoses: Vec<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl IrrigationSchedule {
    pub async fn create(
        schedule_name: String,
        schedule_start_time: NaiveDateTime,
        schedule_days_of_week: Vec<DayOfWeek>,
        db: DbPool,
    ) -> Result<(), Error> {
        web::block(move || {
            let mut conn = db.get().expect("Could not get a db connection.");

            diesel::insert_into(irrigation_schedule::table)
                .values(
                    name.eq(schedule_name), //hose_id.eq(hose_id),
                                            //status.eq(IrrigationEventStatus::InProgress),
                )
                .execute(&mut conn)
        })
        .await?
        .map_err(|e| anyhow!("Internal server error when creating irrigation schedule: {e}"))?;

        Ok(())
    }

    pub async fn delete(id: i32, db: DbPool) -> Result<(), Error> {
        web::block(move || {
            let mut conn = db.get().expect("Could not get a db connection.");

            diesel::delete(irrigation_schedule::table)
                .execute(&mut conn)
                .map_err(|e| anyhow!("Error deleting irrigation schedules: {}", e))
        })
        .await?
        .map_err(|e| anyhow!("Internal server error when deleting irrigation schedule: {e}"))?;

        Ok(())
    }

    pub async fn edit(
        schedule_id: i32,
        schedule_name: String,
        schedule_start_time: NaiveDateTime,
        schedule_days_of_week: Vec<DayOfWeek>,
        db: DbPool,
    ) -> Result<(), Error> {
        web::block(move || {
            let mut conn = db.get().expect("Could not get a db connection.");

            let existing_event = irrigation_schedule::table
                .find(id)
                .first::<IrrigationSchedule>(&mut conn)
                .optional()
                .map_err(|e| anyhow!("Error checking for existing irrigation event: {}", e))?;

            if let Some(event) = existing_event {
                let updated_event = IrrigationSchedule {
                    name: name.unwrap_or_else(|| event.name),
                    start_time: start_time.unwrap_or_else(|| event.start_time),
                    days_of_week: days_of_week.unwrap_or_else(|| event.days_of_week),
                    ..event
                };

                diesel::update(irrigation_schedule::table.find(id))
                    .set(&updated_event)
                    .execute(&mut conn)
                    .map_err(|e| anyhow!("Error updating irrigation event: {}", e))?;
            } else {
                return Err(anyhow!("No irrigation event found with ID {}", id));
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
        irrigation_schedule::table
            .select(IrrigationSchedule::as_select())
            .limit(100)
    }
}
