use std::fmt;

use anyhow::{anyhow, Error};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::query_builder::SqlQuery;
use diesel::sql_query;
use diesel::sql_types::{Bool, Integer, Nullable, Text};
use diesel::sqlite::Sqlite;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::hydro::schedule::Status;
use crate::schema::irrigation_event::*;
use crate::schema::{irrigation_event, irrigation_schedule};
use crate::util::spawn_blocking_with_tracing;

type BoxedQuery<'a> = irrigation_event::BoxedQuery<'a, Sqlite, irrigation_event::SqlType>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum IrrigationEventStatus {
    Cancelled,
    Completed,
    InProgress,
    Queued,
}

#[derive(Clone, Debug, Insertable, PartialEq, Queryable, Selectable, Serialize, Deserialize)]
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

#[derive(Clone, Debug, QueryableByName)]
pub struct StatusQueryResult {
    #[diesel(sql_type = Integer)]
    pub schedule_schedule_id: i32,
    #[diesel(sql_type = Bool)]
    pub schedule_active: bool,
    #[diesel(sql_type = Integer)]
    pub schedule_duration: i32,
    #[diesel(sql_type = Text)]
    pub schedule_name: String,
    #[diesel(sql_type = Text)]
    pub schedule_start_time: String,
    #[diesel(sql_type = Text)]
    pub schedule_days_of_week: String,
    #[diesel(sql_type = Text)]
    pub schedule_hoses: String,
    #[diesel(sql_type = Text)]
    pub schedule_created_at: String,
    #[diesel(sql_type = Text)]
    pub schedule_updated_at: String,
    #[diesel(sql_type = Nullable<Integer>)]
    pub event_id: Option<i32>,
    #[diesel(sql_type = Nullable<Integer>)]
    pub event_hose_id: Option<i32>,
    #[diesel(sql_type = Nullable<Text>)]
    pub event_status: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub event_created_at: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub event_end_time: Option<String>,
}

impl fmt::Display for IrrigationEventStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IrrigationEventStatus::InProgress => write!(f, "in_progress"),
            IrrigationEventStatus::Completed => write!(f, "completed"),
            IrrigationEventStatus::Cancelled => write!(f, "cancelled"),
            IrrigationEventStatus::Queued => write!(f, "queued"),
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

    // #[tracing::instrument(skip(db))]
    // pub async fn next_queued(db: dyn DbPool) -> Result<(i32, IrrigationEvent), Error> {
    //     let mut conn = match db.get_conn() {
    //         Ok(conn) => conn,
    //         Err(e) => return Err(anyhow!(e)),
    //     };

    //     // TODO: handle not found
    //     let thread_result = spawn_blocking_with_tracing(move || {
    //         irrigation_event::table
    //             .inner_join(irrigation_schedule::table)
    //             .filter(status.eq("queued"))
    //             .select((irrigation_schedule::duration, irrigation_event::all_columns))
    //             .order(created_at.asc())
    //             .first::<(i32, IrrigationEvent)>(&mut conn)
    //             .map_err(|e| anyhow!("Error getting next queued event: {}", e))
    //     })
    //     .await;

    //     let result = match thread_result {
    //         Ok(result) => result,
    //         Err(e) => return Err(anyhow!(e)),
    //     };

    //     match result {
    //         Ok(event) => Ok(event),
    //         Err(e) => Err(anyhow!(e)),
    //     }
    // }

    pub fn status_query() -> SqlQuery {
        sql_query(
            "SELECT
            schedule.id,
            schedule.active,
            schedule.duration,
            schedule.name,
            schedule.status,
            schedule.start_time,
            schedule.days_of_week,
            schedule.hoses,
            schedule.created_at,
            schedule.updated_at,
            event.id,
            event.hose_id,
            event.status,
            event.created_at,
            event.end_time
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

    // #[tracing::instrument(skip(db))]
    // pub async fn create<D>(hose: i32, schedule: i32, db: impl DbPool) -> Result<(), Error>
    // where
    //     D: DbPool + ?Sized + 'static,
    // {
    //     spawn_blocking_with_tracing(move || {
    //         let mut conn = db.get_conn().expect("Could not get a db connection.");

    //         let existing_event = IrrigationEvent::in_progress()
    //             .first::<IrrigationEvent>(&mut conn)
    //             .optional()
    //             .map_err(|e| anyhow!("Error checking for existing irrigation event: {}", e))?;

    //         if existing_event.is_some() {
    //             return Err(anyhow!("An irrigation event is already in progress."));
    //         };

    //         diesel::insert_into(irrigation_event::table)
    //             .values((
    //                 schedule_id.eq(schedule),
    //                 hose_id.eq(hose),
    //                 status.eq(IrrigationEventStatus::InProgress.to_string()),
    //             ))
    //             .execute(&mut conn)
    //             .map_err(|e| anyhow!("Error creating irrigation event: {}", e))
    //     })
    //     .await?
    //     .map_err(|e| anyhow!("Internal server error when creating irrigation event: {e}"))?;

    //     Ok(())
    // }

    // pub async fn create_irrigation_events_for_status(
    //     db: impl DbPool,
    //     stat: Status,
    // ) -> Result<usize, Error> {
    //     let mut conn = match db.get_conn() {
    //         Ok(conn) => conn,
    //         Err(e) => return Err(anyhow!(e)),
    //     };

    //     let hoses: Vec<i32> = stat
    //         .schedule
    //         .hoses
    //         .split(",")
    //         .map(|hose| {
    //             hose.parse::<i32>().unwrap_or_else(|err| {
    //                 error!("Failed to parse hose: {}", err);
    //                 0
    //             })
    //         })
    //         .filter(|hose| hose > &0)
    //         .collect();

    //     let events_to_queue: Vec<_> = hoses
    //         .into_iter()
    //         .map(|hose| {
    //             (
    //                 hose_id.eq(hose),
    //                 schedule_id.eq(stat.schedule.id),
    //                 status.eq(IrrigationEventStatus::Queued.to_string()),
    //             )
    //         })
    //         .collect();

    //     let rows_updated: usize = spawn_blocking_with_tracing(move || {
    //         diesel::insert_into(irrigation_event::table)
    //             .values(events_to_queue)
    //             .execute(&mut conn)
    //             .map_err(|e| anyhow!(e))
    //     })
    //     .await??;

    //     Ok(rows_updated)
    // }

    // #[tracing::instrument(skip(db))]
    // pub async fn finish(db: impl DbPool) -> Result<bool, Error> {
    //     let mut conn = db.get_conn().expect("Could not get a db connection.");

    //     spawn_blocking_with_tracing(move || {
    //         let events: Vec<IrrigationEvent> = irrigation_event::table
    //             .filter(status.eq(IrrigationEventStatus::InProgress.to_string()))
    //             .select(IrrigationEvent::as_select())
    //             .load(&mut conn)
    //             .expect("Error loading in_progress irrigation events.");

    //         let event = match events.len() {
    //             0 => return Ok(false),
    //             1 => events.first().unwrap(),
    //             _ => return Err(anyhow!("Multiple in_progress events found.")),
    //         };

    //         let status_result = diesel::update(irrigation_event::table.find(event.id))
    //             .set(status.eq(IrrigationEventStatus::InProgress.to_string()))
    //             .execute(&mut conn);

    //         match status_result {
    //             Ok(_) => Ok(true),
    //             Err(e) => Err(anyhow!("Error updating irrigation event: {}", e)),
    //         }
    //     })
    //     .await?
    // }
}
