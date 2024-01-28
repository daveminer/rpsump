use chrono::{NaiveDateTime, NaiveTime};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::schema::irrigation_schedule;

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
    pub active: bool,
    pub name: String,
    pub duration: i32,
    pub start_time: NaiveTime,
    pub days_of_week: String,
    pub hoses: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, serde::Deserialize)]
pub struct CreateIrrigationScheduleParams {
    pub active: bool,
    pub days_of_week: Vec<DayOfWeek>,
    pub hoses: Vec<i32>,
    pub name: String,
    pub duration: i32,
    pub start_time: NaiveTime,
}

#[derive(Debug, serde::Deserialize)]
pub struct UpdateIrrigationScheduleParams {
    pub active: Option<bool>,
    pub days_of_week: Option<Vec<DayOfWeek>>,
    pub hoses: Option<Vec<i32>>,
    pub name: Option<String>,
    pub duration: Option<i32>,
    pub start_time: Option<NaiveTime>,
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
