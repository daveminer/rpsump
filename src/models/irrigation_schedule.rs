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

enum DayOfWeek {
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
        name: String,
        start_time: NaiveDateTime,
        days_of_week: Vec<DayOfWeek>,
        db: DbPool,
    ) -> Result<(), Error> {
        web::block(move || {
            let mut conn = db.get().expect("Could not get a db connection.");

            diesel::insert_into(irrigation_event::table)
                .values((
                    hose_id.eq(hose_id),
                    status.eq(IrrigationEventStatus::InProgress),
                ))
                .execute(&mut conn)
        })
        .await?
        .map_err(|e| anyhow!("Internal server error when creating irrigation schedule: {e}"))?;

        Ok(())
    }

    pub async fn delete() -> Result<(), Error> {
        web::block(move || {
            let mut conn = db.get().expect("Could not get a db connection.");

            diesel::delete(irrigation_schedules::table)
                .execute(&mut conn)
                .map_err(|e| anyhow!("Error deleting irrigation schedules: {}", e))
        })
        .await?
        .map_err(|e| anyhow!("Internal server error when deleting irrigation schedule: {e}"))?;

        Ok(())
    }

    pub async fn edit(
        id: i32,
        name: Option<String>,
        start_time: Option<NaiveDateTime>,
        days_of_week: Option<Vec<DayOfWeek>>,
        db: DbPool,
    ) -> Result<(), Error> {
        web::block(move || {
            let mut conn = db.get().expect("Could not get a db connection.");

            let existing_event = irrigation_event::table
                .find(id)
                .first::<IrrigationEvent>(&mut conn)
                .optional()
                .map_err(|e| anyhow!("Error checking for existing irrigation event: {}", e))?;

            if let Some(event) = existing_event {
                let updated_event = IrrigationEvent {
                    name: name.unwrap_or_else(|| event.name),
                    start_time: start_time.unwrap_or_else(|| event.start_time),
                    days_of_week: days_of_week.unwrap_or_else(|| event.days_of_week),
                    ..event
                };

                diesel::update(irrigation_event::table.find(id))
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

    pub fn all<DB>() -> Select<sump_event::table, AsSelect<IrrigationEvent, DB>>
    where
        DB: Backend,
    {
        irrigation_event::table
            .select(IrrigationEvent::as_select())
            .limit(100)
    }

    pub fn in_progress<DB>(
        conn: &DB::Connection,
        schedule_id: i32,
    ) -> Result<bool, diesel::result::Error>
    where
        DB: diesel::backend::Backend,
    {
        use diesel::dsl::exists;

        let result = irrigation_event::table
            .filter(irrigation_event::schedule_id.eq(schedule_id))
            .filter(irrigation_event::status.eq(IrrigationEventStatus::InProgress))
            .select(exists(irrigation_event::table))
            .get_result(conn)?;

        Ok(result)
    }
}
