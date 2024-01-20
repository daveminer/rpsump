use actix_web::web::Data;
use anyhow::{anyhow, Error};
use chrono::{NaiveDateTime, NaiveTime};
use diesel::prelude::*;
use diesel::sqlite::Sqlite;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::database::{DbConn, DbPool};
use crate::schema::irrigation_schedule;
use crate::schema::irrigation_schedule::*;
use crate::util::spawn_blocking_with_tracing;

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

type BoxedQuery<'a> = irrigation_schedule::BoxedQuery<'a, Sqlite, irrigation_schedule::SqlType>;

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

impl IrrigationSchedule {
    // Composable queries
    pub fn active() -> BoxedQuery<'static> {
        IrrigationSchedule::all().filter(irrigation_schedule::active.eq(true))
    }

    pub fn all() -> BoxedQuery<'static> {
        irrigation_schedule::table.limit(100).into_boxed()
    }

    pub fn by_id(sched_id: i32) -> BoxedQuery<'static> {
        irrigation_schedule::table.find(sched_id).into_boxed()
    }

    pub fn by_user_id(user_id: i32) -> BoxedQuery<'static> {
        irrigation_schedule::table
            .filter(irrigation_schedule::id.eq(user_id))
            .into_boxed()
    }

    #[tracing::instrument(skip(db))]
    pub async fn create<D>(
        schedule_hoses: Vec<i32>,
        schedule_name: String,
        schedule_start_time: NaiveTime,
        schedule_duration: i32,
        schedule_days_of_week: Vec<DayOfWeek>,
        db: Data<D>,
    ) -> Result<IrrigationSchedule, Error>
    where
        D: DbPool + 'static + ?Sized,
    {
        let hose_str = schedule_hoses
            .iter()
            .map(|hose| hose.to_string())
            .collect::<Vec<String>>()
            .join(",");

        let day_of_week_str = schedule_days_of_week
            .iter()
            .map(|day| day.to_string())
            .collect::<Vec<String>>()
            .join(",");

        let new_schedule = spawn_blocking_with_tracing(move || {
            let mut conn = db.get_conn().expect("Could not get a db connection.");

            return diesel::insert_into(irrigation_schedule::table)
                .values((
                    active.eq(true),
                    hoses.eq(hose_str),
                    name.eq(schedule_name),
                    start_time.eq(schedule_start_time),
                    duration.eq(schedule_duration),
                    days_of_week.eq(day_of_week_str),
                ))
                .get_result::<IrrigationSchedule>(&mut conn);
        })
        .await
        .map_err(|e| anyhow!("Error: {e}"))?
        .map_err(|e| anyhow!("Internal server error when creating irrigation schedule: {e}"))?;

        Ok(new_schedule)
    }

    #[tracing::instrument(skip(db))]
    pub async fn delete<D>(schedule_id: i32, db: Data<D>) -> Result<IrrigationSchedule, Error>
    where
        D: DbPool + 'static + ?Sized,
    {
        let mut conn = db.get_conn()?;

        let result = spawn_blocking_with_tracing(move || {
            diesel::delete(irrigation_schedule::table.find(schedule_id))
                .get_result::<IrrigationSchedule>(&mut conn)
                .map_err(|e| anyhow!("Error deleting irrigation schedules: {}", e))
        })
        .await??;

        Ok(result)
    }

    #[tracing::instrument(skip(db))]
    pub async fn edit<D>(
        schedule_id: i32,
        schedule_hoses: Option<Vec<i32>>,
        schedule_name: Option<String>,
        schedule_start_time: Option<NaiveTime>,
        schedule_days_of_week: Option<Vec<DayOfWeek>>,
        db: Data<D>,
    ) -> Result<Option<IrrigationSchedule>, Error>
    where
        D: DbPool + 'static + ?Sized,
    {
        let mut conn = db.get_conn()?;
        spawn_blocking_with_tracing(move || {
            conn.transaction::<_, Error, _>(|conn| {
                update_schedule(
                    conn,
                    schedule_id,
                    schedule_hoses,
                    schedule_name,
                    schedule_start_time,
                    schedule_days_of_week,
                )
            })
        })
        .await?
    }
}

#[tracing::instrument(skip(conn))]
fn update_schedule(
    conn: &mut DbConn,
    schedule_id: i32,
    schedule_hoses: Option<Vec<i32>>,
    schedule_name: Option<String>,
    schedule_start_time: Option<NaiveTime>,
    schedule_days_of_week: Option<Vec<DayOfWeek>>,
) -> Result<Option<IrrigationSchedule>, Error> {
    let existing_schedule_query = IrrigationSchedule::by_id(schedule_id)
        .first::<IrrigationSchedule>(conn)
        .optional();

    let mut new = match existing_schedule_query {
        Ok(Some(existing_schedule)) => existing_schedule,
        Ok(None) => return Ok(None),
        Err(e) => {
            return Err(anyhow!(
                "Could not check for existing irrigation schedule: {}",
                e
            ))
        }
    };

    if let Some(schedule_hoses) = schedule_hoses {
        new.hoses = schedule_hoses
            .iter()
            .map(|hose| hose.to_string())
            .collect::<Vec<String>>()
            .join(",");
    }

    if let Some(schedule_name) = schedule_name {
        new.name = schedule_name;
    }

    if let Some(schedule_start_time) = schedule_start_time {
        new.start_time = schedule_start_time;
    }

    if let Some(schedule_days_of_week) = schedule_days_of_week {
        new.days_of_week = schedule_days_of_week
            .iter()
            .map(|day| day.to_string())
            .collect::<Vec<String>>()
            .join(",");
    }

    let result: Result<Option<IrrigationSchedule>, Error> =
        diesel::update(irrigation_schedule::table.find(schedule_id))
            .set(new)
            .get_result::<IrrigationSchedule>(conn)
            .optional()
            .map_err(|e| anyhow!("Error updating irrigation event: {}", e));

    match result {
        Ok(None) => return Err(anyhow!("Irrigation schedule not found.")),
        Ok(schedule) => Ok(schedule),
        Err(e) => Err(anyhow!("Error updating irrigation event: {}", e)),
    }
}
