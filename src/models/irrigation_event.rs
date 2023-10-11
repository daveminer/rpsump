use std::fmt;

use actix_web::web;
use anyhow::{anyhow, Error};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::query_builder::SqlQuery;
use diesel::result::Error as DieselError;
use diesel::sql_query;
use diesel::sqlite::Sqlite;
use serde::{Deserialize, Serialize};

use crate::database::DbPool;
use crate::schema::irrigation_event;
use crate::schema::irrigation_event::*;
use crate::sump::schedule::Status;

type BoxedQuery<'a> = irrigation_event::BoxedQuery<'a, Sqlite, irrigation_event::SqlType>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum IrrigationEventStatus {
    Cancelled,
    Completed,
    InProgress,
}

#[derive(Clone, Debug, PartialEq, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(belongs_to(IrrigationSchedule))]
#[diesel(table_name = irrigation_event)]
pub struct IrrigationEvent {
    pub id: i32,
    pub hose_id: i32,
    pub created_at: NaiveDateTime,
    pub end_time: Option<NaiveDateTime>,
    pub status: String,
    pub schedule_id: i32,
}

impl fmt::Display for IrrigationEventStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IrrigationEventStatus::InProgress => write!(f, "in_progress"),
            IrrigationEventStatus::Completed => write!(f, "completed"),
            IrrigationEventStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl IrrigationEvent {
    // Composable queries
    pub fn all() -> BoxedQuery<'static> {
        irrigation_event::table.limit(100).into_boxed()
    }

    pub fn by_id(event_id: i32) -> BoxedQuery<'static> {
        irrigation_event::table
            .filter(irrigation_event::id.eq(event_id))
            .into_boxed()
    }

    pub fn for_schedule(sched_id: i32) -> BoxedQuery<'static> {
        irrigation_event::table
            .filter(irrigation_event::schedule_id.eq(sched_id))
            .into_boxed()
    }

    pub fn in_progress() -> BoxedQuery<'static> {
        irrigation_event::table
            .filter(status.eq("in_progress"))
            .into_boxed()
    }

    pub fn finished() -> BoxedQuery<'static> {
        irrigation_event::table
            .filter(status.eq("completed"))
            .into_boxed()
    }

    pub fn irrigation_running() -> BoxedQuery<'static> {
        irrigation_event::table
            .filter(status.eq("in_progress"))
            .into_boxed()
    }

    pub fn status_query() -> SqlQuery {
        sql_query(
            "SELECT schedule.id, schedule.status, schedule.start_time, event.id, event.status, event.created_at, event.end_time
            FROM irrigation_schedule AS schedule
            LEFT JOIN (
                SELECT *
                FROM (
                    SELECT *,
                        ROW_NUMBER() OVER (
                            PARTITION BY schedule_id, hose_id
                            ORDER BY created_at DESC
                        ) AS row_number
                    FROM irrigation_event
                ) AS event
                WHERE events.row_number = 1
            ) AS events ON events.schedule_id = schedules.id
            WHERE schedules.status = 'active'",
        )
    }

    pub async fn create(hose: i32, schedule: i32, db: DbPool) -> Result<(), Error> {
        web::block(move || {
            let mut conn = db.get().expect("Could not get a db connection.");

            let existing_event = irrigation_event::table
                .filter(status.eq("in_progress"))
                .first::<IrrigationEvent>(&mut conn)
                .optional()
                .map_err(|e| anyhow!("Error checking for existing irrigation event: {}", e))?;

            if existing_event.is_some() {
                return Err(anyhow!("An irrigation event is already in progress."));
            };

            diesel::insert_into(irrigation_event::table)
                .values((
                    schedule_id.eq(schedule),
                    hose_id.eq(hose),
                    status.eq(IrrigationEventStatus::InProgress.to_string()),
                ))
                .execute(&mut conn)
                .map_err(|e| anyhow!("Error creating irrigation event: {}", e))
        })
        .await?
        .map_err(|e| anyhow!("Internal server error when creating irrigation event: {e}"))?;

        Ok(())
    }

    pub fn create_irrigation_event(db: DbPool, status_to_run: Status) -> Result<(), Error> {
        let mut conn: diesel::r2d2::PooledConnection<
            diesel::r2d2::ConnectionManager<SqliteConnection>,
        > = db.get().expect("Could not get a db connection.");
        // Check if there is an in-progress event for this schedule
        let result = conn.transaction(|conn| {
            let in_progress_events =
                IrrigationEvent::in_progress().get_results::<IrrigationEvent>(conn);

            match in_progress_events {
                Ok(events) => {
                    if events.len() > 0 {
                        // An in-progress event already exists, so roll back the transaction
                        return Err(DieselError::RollbackTransaction);
                    }
                }
                Err(e) => return Err(DieselError::RollbackTransaction),
            };

            // diesel::insert_into(irrigation_event::table)
            //     .values((
            //         hose_id.eq(status.hose_id),
            //         schedule_id.eq(sched_id),
            //         status.eq(IrrigationEventStatus::InProgress.to_string()),
            //     ))
            //     .execute(conn)?;

            Ok(())
        });

        Ok(())
    }

    pub async fn finish(db: DbPool) -> Result<bool, Error> {
        let mut conn = db.get().expect("Could not get a db connection.");

        web::block(move || {
            let events: Vec<IrrigationEvent> = irrigation_event::table
                .filter(status.eq(IrrigationEventStatus::InProgress.to_string()))
                .select(IrrigationEvent::as_select())
                .load(&mut conn)
                .expect("Error loading in_progress irrigation events.");

            let event = match events.len() {
                0 => return Ok(false),
                1 => events.first().unwrap(),
                _ => return Err(anyhow!("Multiple in_progress events found.")),
            };

            let status_result = diesel::update(irrigation_event::table.find(event.id))
                .set(status.eq(IrrigationEventStatus::InProgress.to_string()))
                .execute(&mut conn);

            match status_result {
                Ok(_) => Ok(true),
                Err(e) => Err(anyhow!("Error updating irrigation event: {}", e)),
            }
        })
        .await?
    }
}
