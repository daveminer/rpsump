use anyhow::{anyhow, Error};
use chrono::{Datelike, Duration, NaiveDateTime};
use diesel::prelude::*;
use diesel::sql_types::{Bool, Integer, Text};
use diesel::RunQueryDsl;
use std::thread::{self, sleep, JoinHandle};
use std::time::Duration as StdDuration;

use crate::controllers::spawn_blocking_with_tracing;
use crate::email::sendinblue::send_error_email;
use crate::models::irrigation_event::{IrrigationEventRunError, IrrigationEventStatus};
use crate::{database::DbPool, models::irrigation_event::IrrigationEvent};

#[derive(Clone, Debug, Queryable, QueryableByName)]
pub struct Status {
    #[diesel(sql_type = Integer)]
    pub schedule_id: i32,
    #[diesel(sql_type = Text)]
    pub schedule_days_of_week: String,
    #[diesel(sql_type = Text)]
    pub schedule_start_time: String,
    #[diesel(sql_type = Bool)]
    pub schedule_status: bool,
    #[diesel(sql_type = Integer)]
    pub event_id: i32,
    #[diesel(sql_type = Integer)]
    pub event_hose_id: i32,
    #[diesel(sql_type = Text)]
    pub event_status: String,
    #[diesel(sql_type = Text)]
    pub event_created_at: String,
    #[diesel(sql_type = Text)]
    pub event_end_time: String,
}

pub fn start(
    db: DbPool,
    mailer_url: &str,
    contact_email: &str,
    auth_token: &str,
) -> JoinHandle<()> {
    let auth_token_clone = auth_token.to_string();
    let contact_email_clone = contact_email.to_string();
    let db_clone = db.clone();
    let mailer_url_clone = mailer_url.to_string();

    spawn_blocking_with_tracing(move || loop {
        sleep(StdDuration::from_secs(5));

        let statuses = match schedule_status(db_clone.clone()) {
            Ok(statuses) => statuses,
            Err(e) => {
                error_email(
                    db_clone.clone(),
                    &mailer_url_clone,
                    &contact_email_clone,
                    &auth_token_clone,
                    e,
                );

                continue;
            }
        };

        match schedule_tick(statuses) {
            Ok(statuses) => {
                if statuses.len() == 0 {
                    continue;
                }

                // Run a schedule
                schedule_run(db_clone.clone(), statuses);
            }
            Err(e) => error_email(
                db_clone.clone(),
                &mailer_url_clone,
                &contact_email_clone,
                &auth_token_clone,
                e,
            ),
        }
    })
}

fn schedule_status(db: DbPool) -> Result<Vec<Status>, Error> {
    let mut conn = match db.get() {
        Ok(conn) => conn,
        Err(e) => return Err(e.into()),
    };

    IrrigationEvent::status_query()
        .load(&mut conn)
        .map_err(|e| anyhow!(e))
}

fn schedule_tick(status_list: Vec<Status>) -> Result<Vec<Status>, Error> {
    match schedule_running(&status_list) {
        Ok(true) => return Ok(vec![]),
        Ok(false) => Ok(schedules_due(status_list)),
        Err(e) => return Err(e),
    }
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

fn schedule_run(db: DbPool, statuses: Vec<Status>) {
    // Create an event for the first schedule in the list
    let new_event: <Result<usize, IrrigationEventRunError>> =
        IrrigationEvent::create_irrigation_event(db, statuses[0].clone());

    // Start the irrigation I/O in a new thread
    let irrigation_job = start_irrigation();

    Ok((new_event, irrigation_job))
}

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

fn start_irrigation() -> JoinHandle<()> {
    thread::spawn(|| {
        // Exit if water is too low
        // Open the solenoid for the job
        // Start the pump

        // Wait for the job to finish
        // Stop the pump
        // Close the solenoid
        // Move the job out of "in progress" status
        })
}

fn error_email(db: DbPool, mailer_url: &str, contact_email: &str, auth_token: &str, e: Error) {
    futures::executor::block_on(async {
        let result =
            send_error_email(db, mailer_url, contact_email, auth_token, &e.to_string()).await;

        match result {
            Ok(_) => (),
            Err(e) => tracing::error!("Could not send error email: {:?}", e),
        }
    })
}

#[tokio::test]
async fn schedules_due_ready() {
    let status = Status {
        schedule_id: 1,
        schedule_days_of_week: "1,2,3,4,5,6,7".to_string(),
        schedule_start_time: "2021-01-01 00:00:00".to_string(),
        schedule_status: true,
        event_id: 1,
        event_hose_id: 1,
        event_status: IrrigationEventStatus::InProgress.to_string(),
        event_created_at: "2021-01-01 00:00:00".to_string(),
        event_end_time: "2021-01-01 00:00:00".to_string(),
    };

    let result = schedules_due(vec![status]);
    println!("RESULT: {:?}", result);
}
