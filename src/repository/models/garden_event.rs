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
    /// The schedule's name as of the moment the event was queued. Kept on the
    /// row so history stays readable after a rename or a delete (the
    /// `schedule_id` FK is `ON DELETE SET NULL`). `None` for manual runs.
    pub schedule_name: Option<String>,
}

#[derive(Clone, Debug, Insertable)]
#[diesel(table_name = garden_event)]
pub struct NewGardenEvent {
    pub schedule_id: Option<i32>,
    pub source: String,
    pub status: String,
    pub scheduled_for: NaiveDateTime,
    pub duration_secs: i32,
    pub schedule_name: Option<String>,
}

/// Filters for `Repository::garden_events`. `from`/`to` bound `scheduled_for`,
/// which is set for every event (including skipped and manual ones).
#[derive(Clone, Debug, Default)]
pub struct GardenEventFilter {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub source: Option<GardenEventSource>,
    pub status: Option<GardenEventStatus>,
    pub from: Option<NaiveDateTime>,
    pub to: Option<NaiveDateTime>,
}

impl GardenEventFilter {
    /// Every event from `from` onward, newest first, without pagination. The
    /// upper edge is deliberately left open: a rollup decides its own `to`
    /// after the query returns, so a run that starts mid-request is not lost
    /// between the two.
    pub fn since(from: NaiveDateTime) -> Self {
        Self {
            from: Some(from),
            ..Default::default()
        }
    }
}

impl GardenEvent {
    pub fn parsed_status(&self) -> Option<GardenEventStatus> {
        GardenEventStatus::from_str(&self.status).ok()
    }

    pub fn parsed_source(&self) -> Option<GardenEventSource> {
        GardenEventSource::from_str(&self.source).ok()
    }

    /// When the event actually happened, falling back to when it was meant to.
    pub fn occurred_at(&self) -> NaiveDateTime {
        self.start_time.unwrap_or(self.scheduled_for)
    }

    /// Seconds of water actually delivered. A completed or cancelled run is
    /// measured from its own timestamps, so a run stopped early counts only
    /// what it used; a run still in progress is measured against `now`.
    /// Queued and skipped events delivered nothing.
    pub fn watered_secs(&self, now: NaiveDateTime) -> i64 {
        match self.parsed_status() {
            Some(GardenEventStatus::Completed) => match (self.start_time, self.end_time) {
                (Some(start), Some(end)) => (end - start).num_seconds().max(0),
                // A completed event with no timestamps predates this
                // bookkeeping; fall back to what it was asked to run.
                _ => self.duration_secs.max(0) as i64,
            },
            // A cancelled event that never started (stopped while queued)
            // delivered nothing, so it must not fall back to duration_secs.
            Some(GardenEventStatus::Cancelled) => match (self.start_time, self.end_time) {
                (Some(start), Some(end)) => (end - start).num_seconds().max(0),
                _ => 0,
            },
            Some(GardenEventStatus::InProgress) => match self.start_time {
                Some(start) => (now - start).num_seconds().clamp(0, self.duration_secs.max(0) as i64),
                None => 0,
            },
            _ => 0,
        }
    }

    /// Seconds this event would have run had the rain not skipped it.
    pub fn skipped_secs(&self) -> i64 {
        match self.parsed_status() {
            Some(GardenEventStatus::Skipped) => self.duration_secs.max(0) as i64,
            _ => 0,
        }
    }
}
