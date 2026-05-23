use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::schema::garden_event;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GardenEventStatus {
    Queued,
    InProgress,
    Completed,
    Cancelled,
    Skipped,
}

impl fmt::Display for GardenEventStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GardenEventStatus::Queued => write!(f, "queued"),
            GardenEventStatus::InProgress => write!(f, "in_progress"),
            GardenEventStatus::Completed => write!(f, "completed"),
            GardenEventStatus::Cancelled => write!(f, "cancelled"),
            GardenEventStatus::Skipped => write!(f, "skipped"),
        }
    }
}

impl FromStr for GardenEventStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "queued" => Ok(GardenEventStatus::Queued),
            "in_progress" => Ok(GardenEventStatus::InProgress),
            "completed" => Ok(GardenEventStatus::Completed),
            "cancelled" => Ok(GardenEventStatus::Cancelled),
            "skipped" => Ok(GardenEventStatus::Skipped),
            other => Err(format!("unknown garden event status: {}", other)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GardenEventSource {
    Scheduled,
    Manual,
}

impl fmt::Display for GardenEventSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GardenEventSource::Scheduled => write!(f, "scheduled"),
            GardenEventSource::Manual => write!(f, "manual"),
        }
    }
}

impl FromStr for GardenEventSource {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "scheduled" => Ok(GardenEventSource::Scheduled),
            "manual" => Ok(GardenEventSource::Manual),
            other => Err(format!("unknown garden event source: {}", other)),
        }
    }
}

#[derive(
    Clone, Debug, Identifiable, PartialEq, Queryable, Selectable, Serialize, Deserialize,
)]
#[diesel(table_name = garden_event)]
pub struct GardenEvent {
    pub id: i32,
    pub schedule_id: Option<i32>,
    pub source: String,
    pub status: String,
    pub scheduled_for: NaiveDateTime,
    pub duration_secs: i32,
    pub start_time: Option<NaiveDateTime>,
    pub end_time: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

#[derive(Clone, Debug, Insertable)]
#[diesel(table_name = garden_event)]
pub struct NewGardenEvent {
    pub schedule_id: Option<i32>,
    pub source: String,
    pub status: String,
    pub scheduled_for: NaiveDateTime,
    pub duration_secs: i32,
}
