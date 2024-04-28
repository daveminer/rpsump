use chrono::{Duration, NaiveDateTime, Utc};
use diesel::sql_types::{Bool, Integer, Nullable, Text};
use diesel::{prelude::*, query_builder::SqlQuery, sql_query, sqlite::Sqlite};
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::schema::irrigation_event;

type BoxedQuery<'a> = irrigation_event::BoxedQuery<'a, Sqlite, irrigation_event::SqlType>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]

pub enum IrrigationEventStatus {
    Cancelled,
    Completed,
    InProgress,
    Queued,
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

#[derive(Clone, Debug, Insertable, PartialEq, Serialize, Deserialize)]
#[diesel(belongs_to(IrrigationSchedule))]
#[diesel(table_name = irrigation_event)]
pub struct NewIrrigationEvent {
    pub hose_id: i32,
    pub created_at: NaiveDateTime,
    pub end_time: Option<NaiveDateTime>,
    pub status: String,
    pub schedule_id: i32,
}

#[derive(Clone, Debug, QueryableByName)]
pub struct StatusQueryResult {
    #[diesel(sql_type = Integer)]
    pub id: i32,
    #[diesel(sql_type = Bool)]
    pub active: bool,
    #[diesel(sql_type = Integer)]
    pub duration: i32,
    #[diesel(sql_type = Text)]
    pub name: String,
    #[diesel(sql_type = Text)]
    pub start_time: String,
    #[diesel(sql_type = Text)]
    pub days_of_week: String,
    #[diesel(sql_type = Text)]
    pub hoses: String,
    #[diesel(sql_type = Text)]
    pub created_at: String,
    #[diesel(sql_type = Text)]
    pub updated_at: String,
    #[diesel(sql_type = Nullable<Integer>)]
    pub event_id: Option<i32>,
    #[diesel(sql_type = Nullable<Integer>)]
    pub hose_id: Option<i32>,
    #[diesel(sql_type = Nullable<Text>)]
    pub status: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub end_time: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub event_created_at: Option<String>,
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
    pub fn completed_before(&self, hours: i64) -> bool {
        let now = Utc::now().naive_utc();
        let hours_ago = now - Duration::hours(hours);
        match self.end_time {
            Some(end_time) => end_time < hours_ago,
            None => false,
        }
    }

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
            .filter(irrigation_event::status.eq("in_progress"))
            .into_boxed()
    }

    pub fn status_query() -> SqlQuery {
        sql_query(
            "SELECT
            schedule.id,
            schedule.active,
            schedule.duration,
            schedule.name,
            schedule.start_time,
            schedule.days_of_week,
            schedule.hoses,
            schedule.created_at,
            schedule.updated_at,
            event.id AS event_id,
            event.hose_id,
            event.status,
            event.end_time,
            event.schedule_id,
            event.created_at as event_created_at
            FROM irrigation_schedule AS schedule
            LEFT JOIN (
                SELECT
                    id,
                    hose_id,
                    status,
                    created_at,
                    end_time,
                    schedule_id,
                    ROW_NUMBER() OVER (
                        PARTITION BY schedule_id, hose_id
                        ORDER BY created_at DESC
                    ) AS row_number
                FROM irrigation_event
            ) AS event ON event.schedule_id = schedule.id AND event.row_number = 1
            WHERE schedule.active = true",
        )
    }
}
