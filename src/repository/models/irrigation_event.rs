use chrono::NaiveDateTime;
use diesel::sql_types::{Bool, Integer, Nullable, Text};
use diesel::{prelude::*, query_builder::SqlQuery, sql_query, sqlite::Sqlite};
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::schema::irrigation_event;
use crate::schema::irrigation_event::*;

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
}
