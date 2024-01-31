use chrono::{NaiveDateTime, NaiveTime};
use diesel::prelude::*;
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
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
    #[serde(
        serialize_with = "serialize_hoses",
        deserialize_with = "deserialize_hoses"
    )]
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

fn serialize_hoses<S>(hoses: &String, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let hoses: Vec<i32> = hoses.split(',').map(|s| s.parse().unwrap()).collect();
    serializer.collect_seq(hoses)
}

struct HosesVisitor;

impl<'de> Visitor<'de> for HosesVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a sequence of integers separated by commas")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<String, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let mut hoses = String::new();
        while let Some(value) = seq.next_element::<i32>()? {
            if !hoses.is_empty() {
                hoses.push(',');
            }
            hoses.push_str(&value.to_string());
        }
        Ok(hoses)
    }
}

fn deserialize_hoses<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_seq(HosesVisitor)
}
