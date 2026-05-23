use chrono::{NaiveDateTime, NaiveTime, Weekday};
use diesel::prelude::*;
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::fmt;
use std::str::FromStr;

use crate::schema::garden_schedule;

#[derive(AsChangeset, Clone, Debug, PartialEq, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = garden_schedule)]
pub struct GardenSchedule {
    pub id: i32,
    pub name: String,
    pub active: bool,
    #[serde(
        serialize_with = "serialize_times",
        deserialize_with = "deserialize_times"
    )]
    pub start_times: String,
    #[serde(
        serialize_with = "serialize_days",
        deserialize_with = "deserialize_days"
    )]
    pub days_of_week: String,
    pub duration_secs: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub skip_on_rain: bool,
}

#[derive(Clone, Debug, Insertable)]
#[diesel(table_name = garden_schedule)]
pub struct NewGardenSchedule {
    pub name: String,
    pub active: bool,
    pub start_times: String,
    pub days_of_week: String,
    pub duration_secs: i32,
    pub skip_on_rain: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateGardenScheduleParams {
    pub name: String,
    #[serde(default = "default_true")]
    pub active: bool,
    pub start_times: Vec<NaiveTime>,
    pub days_of_week: Vec<Weekday>,
    pub duration_secs: i32,
    #[serde(default = "default_true")]
    pub skip_on_rain: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateGardenScheduleParams {
    pub name: Option<String>,
    pub active: Option<bool>,
    pub start_times: Option<Vec<NaiveTime>>,
    pub days_of_week: Option<Vec<Weekday>>,
    pub duration_secs: Option<i32>,
    pub skip_on_rain: Option<bool>,
}

fn default_true() -> bool {
    true
}

impl GardenSchedule {
    pub fn parsed_start_times(&self) -> Vec<NaiveTime> {
        parse_times_csv(&self.start_times)
    }

    pub fn parsed_days_of_week(&self) -> Vec<Weekday> {
        parse_days_csv(&self.days_of_week)
    }
}

pub fn times_to_csv(times: &[NaiveTime]) -> String {
    times
        .iter()
        .map(|t| t.format("%H:%M").to_string())
        .collect::<Vec<_>>()
        .join(",")
}

pub fn days_to_csv(days: &[Weekday]) -> String {
    days.iter()
        .map(|d| d.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn parse_times_csv(csv: &str) -> Vec<NaiveTime> {
    csv.split(',')
        .filter_map(|s| {
            let s = s.trim();
            if s.is_empty() {
                None
            } else {
                NaiveTime::parse_from_str(s, "%H:%M")
                    .or_else(|_| NaiveTime::parse_from_str(s, "%H:%M:%S"))
                    .ok()
            }
        })
        .collect()
}

fn parse_days_csv(csv: &str) -> Vec<Weekday> {
    csv.split(',')
        .filter_map(|s| Weekday::from_str(s.trim()).ok())
        .collect()
}

fn serialize_times<S>(value: &str, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let times = parse_times_csv(value);
    serializer.collect_seq(times.iter().map(|t| t.format("%H:%M").to_string()))
}

fn serialize_days<S>(value: &str, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let days = parse_days_csv(value);
    serializer.collect_seq(days.iter().map(|d| d.to_string()))
}

struct TimesVisitor;

impl<'de> Visitor<'de> for TimesVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a sequence of HH:MM time strings")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<String, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let mut out: Vec<String> = Vec::new();
        while let Some(value) = seq.next_element::<String>()? {
            let parsed = NaiveTime::parse_from_str(&value, "%H:%M")
                .or_else(|_| NaiveTime::parse_from_str(&value, "%H:%M:%S"))
                .map_err(de::Error::custom)?;
            out.push(parsed.format("%H:%M").to_string());
        }
        Ok(out.join(","))
    }
}

fn deserialize_times<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_seq(TimesVisitor)
}

struct DaysVisitor;

impl<'de> Visitor<'de> for DaysVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a sequence of weekday names")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<String, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let mut out: Vec<String> = Vec::new();
        while let Some(value) = seq.next_element::<String>()? {
            let parsed = Weekday::from_str(&value).map_err(de::Error::custom)?;
            out.push(parsed.to_string());
        }
        Ok(out.join(","))
    }
}

fn deserialize_days<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_seq(DaysVisitor)
}
