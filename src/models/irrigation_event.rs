use actix_web::web;
use anyhow::{anyhow, Error};
use chrono::NaiveDateTime;
use diesel::backend::Backend;
use diesel::dsl::*;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::database::DbPool;
use crate::schema::irrigation_event;
use crate::schema::irrigation_event::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum IrrigationEventStatus {
    InProgress,
}

#[derive(Clone, Debug, PartialEq, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = irrigation_event)]
pub struct IrrigationEvent {
    pub id: i32,
    pub hose_id: i32,
    pub created_at: NaiveDateTime,
    pub end_time: Option<NaiveDateTime>,
    pub status: IrrigationEventStatus,
    pub schedule_id: i32,
}

impl IrrigationEvent {
    pub async fn create(hose: i32, schedule: i32, db: DbPool) -> Result<(), Error> {
        web::block(move || {
            let mut conn = db.get().expect("Could not get a db connection.");

            let existing_event = irrigation_event::table
                .filter(status.eq(IrrigationEventStatus::InProgress))
                .first::<IrrigationEvent>(&mut conn)
                .optional()
                .map_err(|e| anyhow!("Error checking for existing irrigation event: {}", e))?;

            if existing_event.is_some() {
                return Err(anyhow!(
                    "Irrigation event {} is already in progress.",
                    existing_event.unwrap().id
                ));
            }

            diesel::insert_into(irrigation_event::table)
                .values((
                    hose_id.eq(hose_id),
                    status.eq(IrrigationEventStatus::InProgress),
                ))
                .execute(&mut conn)
                .map_err(|e| anyhow!("Error creating irrigation event: {}", e))
        })
        .await?
        .map_err(|e| anyhow!("Internal server error when creating irrigation event: {e}"))?;

        Ok(())
    }

    pub async fn finish(db: DbPool) -> Result<(), Error> {
        web::block(move || {
            let mut conn = db.get().expect("Could not get a db connection.");

            irrigation_event::table
                .filter(status.eq(IrrigationEventStatus::InProgress))
                .values((
                    hose_id.eq(hose_id),
                    status.eq(IrrigationEventStatus::InProgress),
                ))
                .execute(&mut conn)
        })
        .await?
        .map_err(|e| anyhow!("Internal server error when finishing irrigation event: {e}"))?;

        Ok(())
    }

    pub fn all<DB>() -> Select<irrigation_event::table, AsSelect<IrrigationEvent, DB>>
    where
        DB: Backend,
    {
        irrigation_event::table
            .select(IrrigationEvent::as_select())
            .limit(100)
    }

    pub fn in_progress<DB>() -> Select<irrigation_event::table, AsSelect<IrrigationEvent, DB>>
    where
        DB: Backend,
    {
        let event = irrigation_event::table.filter(status.eq(IrrigationEventStatus::InProgress));

        // Return the first event if only one, else return an error.
        match event.len() {
            0 => Ok(()),
            1 => Ok(event[0]),
            _ => Err("Multiple events found."),
        }
    }
}
