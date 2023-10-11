use crate::models::irrigation_event::IrrigationEventStatus;
use crate::{database::DbPool, models::irrigation_event::IrrigationEvent};
use anyhow::{anyhow, Error};
use chrono::{Datelike, Duration, NaiveDateTime};
use diesel::prelude::*;
use diesel::sql_types::{Bool, Integer, Text};
use diesel::RunQueryDsl;
use std::thread::{self, sleep, JoinHandle};
use std::time::Duration as StdDuration;

#[derive(Clone, Debug, Queryable, QueryableByName)]
pub struct Status {
    #[diesel(sql_type = Integer)]
    schedule_id: i32,
    #[diesel(sql_type = Text)]
    schedule_days_of_week: String,
    #[diesel(sql_type = Text)]
    schedule_start_time: String,
    #[diesel(sql_type = Bool)]
    schedule_status: bool,
    #[diesel(sql_type = Integer)]
    event_id: i32,
    #[diesel(sql_type = Text)]
    event_status: String,
    #[diesel(sql_type = Text)]
    event_created_at: String,
    #[diesel(sql_type = Text)]
    event_end_time: String,
}

pub fn start(db: DbPool) -> JoinHandle<()> {
    thread::spawn(move || {
        poll_irrigation_events(db);
    })
}

fn poll_irrigation_events(db: DbPool) {
    loop {
        sleep(StdDuration::from_secs(5));

        let status_list = match schedule_status(db.clone()) {
            Ok(status_list) => status_list,
            Err(e) => {
                println!("Error getting schedule status: {:?}", e);

                continue;
            }
        };

        match schedule_running(&status_list) {
            Ok(running) => {
                if running {
                    continue;
                }
            }
            Err(e) => {
                println!("Error checking if schedule is running: {:?}", e);
                continue;
            }
        }

        match schedules_due(status_list).as_slice() {
            [] => continue,
            schedules => {
                // Run a schedule
                match create_irrigation_event(db.clone(), schedules[0]) {
                    Ok(()) => (),
                    Err(e) => (), // TODO: report error
                }
                //schedule_run(schedules[0], db.clone())
            }
        }
    }
}

fn schedule_status(db: DbPool) -> Result<Vec<Status>, Error> {
    let mut conn = db.get().expect("Could not get a db connection.");

    IrrigationEvent::status_query()
        .load(&mut conn)
        .map_err(|e| anyhow!(e))
}

fn schedules_due(status_list: Vec<Status>) -> Vec<Status> {
    let now = chrono::Utc::now().naive_utc();

    let mut schedules_to_run = status_list
        .into_iter()
        // Schedule is active
        .filter(|status| status.schedule_status == true)
        // Schedule is for today
        .filter(|status| {
            status
                .schedule_days_of_week
                .contains(&now.weekday().to_string())
        })
        // Schedule's run time has passed
        .filter(|status| status.schedule_start_time.parse::<NaiveDateTime>().unwrap() < now)
        // Last run for this schedule is before today
        .filter(|status| status.event_status == IrrigationEventStatus::InProgress.to_string())
        .collect::<Vec<Status>>();

    schedules_to_run.sort_by(|a, b| a.schedule_start_time.cmp(&b.schedule_start_time));

    return schedules_to_run;
}

// fn schedule_run(db: DbPool, status: Status) {
//     create_irrigation_event(db, status.schedule_id)?;

//     // Check for running event and create the event if it doesn't exist in a transaction
//     let mut conn = db.get().expect("Could not get a db connection.");

//     let result = conn.transaction(|conn| {
//         create_irrigation_event(db, status.schedule_id)?;
//         Ok(())
//     });

//     match result {
//         Ok(_) => {
//             // Transaction succeeded, so start the irrigation event
//         }
//         Err(DieselError::RollbackTransaction) => {
//             // An in-progress event already exists, so do nothing
//         }
//         Err(e) => {
//             // Transaction failed for some other reason, so log the error
//             error!("Failed to create in-progress event: {}", e);
//         }
//     }
// }

fn schedule_running(status_list: &Vec<Status>) -> Result<bool, Error> {
    let active_schedule = status_list
        .iter()
        .find(|status| status.event_status == IrrigationEventStatus::InProgress.to_string());

    if active_schedule.is_none() {
        return Ok(false);
    };

    let active_schedule = active_schedule.unwrap();
    let created_at = match active_schedule.event_created_at.parse::<NaiveDateTime>() {
        Ok(created_at) => created_at,
        Err(e) => return Err(anyhow!("Error parsing created_at: {:?}", e)),
    };
    let now = chrono::Utc::now().naive_utc();
    if now - created_at > Duration::seconds(60) {
        // TODO: Stop event; send error email, create dynamic max runtime variable
    }

    return Ok(true);
}
