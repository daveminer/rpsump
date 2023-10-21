pub mod run;

use anyhow::{anyhow, Error};
use chrono::{Datelike, Duration, NaiveDateTime};
use diesel::prelude::*;
use diesel::sql_types::{Bool, Integer, Text};
use diesel::RunQueryDsl;
use futures::executor::block_on;
use std::time::Duration as StdDuration;
use tokio::task::JoinHandle;
use tokio::time::sleep;

use crate::config::{MailerConfig, Settings};
use crate::controllers::spawn_blocking_with_tracing;
use crate::email::sendinblue::send_error_email;
use crate::models::irrigation_event::IrrigationEventStatus;
use crate::{database::DbPool, models::irrigation_event::IrrigationEvent};

use self::run::run_schedule;

use super::Sump;

#[derive(Clone, Debug, Queryable, QueryableByName)]
pub struct Status {
    #[diesel(sql_type = Integer)]
    pub schedule_id: i32,
    #[diesel(sql_type = Text)]
    pub schedule_days_of_week: String,
    #[diesel(sql_type = Integer)]
    pub schedule_duration: i32,
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

pub fn start(db: DbPool, settings: Settings, sump: Sump) -> JoinHandle<()> {
    let mailer_config = settings.mailer.clone();

    // Schedule logic runs in a new thread
    spawn_blocking_with_tracing(move || {
        loop {
            // Synchronously check for schedules to run
            block_on(sleep(StdDuration::from_secs(5)));

            // Get the statuses of all the schedules
            let statuses = match schedule_status(db.clone()) {
                Ok(statuses) => statuses,
                Err(e) => {
                    error_email(&mailer_config, e);

                    continue;
                }
            };

            // Process any eligible schedules for this instance of the loop
            match schedule_tick(statuses) {
                Ok(statuses) => {
                    if statuses.len() == 0 {
                        continue;
                    }

                    // Run a schedule
                    run_schedule(db.clone(), statuses[0].clone(), sump.clone());
                }
                Err(e) => error_email(&mailer_config, e),
            }
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
    match is_schedule_running(&status_list) {
        Ok(true) => return Ok(vec![]),
        Ok(false) => Ok(due_schedules(status_list, chrono::Utc::now().naive_utc())),
        Err(e) => return Err(e),
    }
}

fn is_schedule_running(status_list: &Vec<Status>) -> Result<bool, Error> {
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

fn due_schedules(status_list: Vec<Status>, now: NaiveDateTime) -> Vec<Status> {
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
        .collect::<Vec<Status>>();

    schedules_to_run.sort_by(|a, b| a.schedule_start_time.cmp(&b.schedule_start_time));

    return schedules_to_run;
}

fn error_email(mailer: &MailerConfig, e: Error) {
    futures::executor::block_on(async {
        let result = send_error_email(mailer, &e.to_string()).await;

        match result {
            Ok(_) => (),
            Err(e) => tracing::error!("Could not send error email: {:?}", e),
        }
    })
}

#[tokio::test]
async fn due_schedules_success() {
    use chrono::Utc;
    let default_noon =
        NaiveDateTime::parse_from_str("2021-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
    //let default_noon = "2021-01-01 00:00:00".to_string();
    let default_all_days = "Monday,Tuesday,Wednesday,Thursday,Friday,Saturday,Sunday".to_string();

    let in_the_past = test_status(
        1,
        default_noon.to_,
        true,
        default_all_days,
        15,
        1,
        1,
        Some((default_noon + Duration::seconds(15)).to_string()),
    );
    let in_the_past = Status {
        schedule_id: 1,
        schedule_days_of_week: "Monday,Tuesday,Wednesday,Thursday,Friday".to_string(),
        schedule_duration: 10,
        schedule_start_time: "2021-01-01 00:00:00".to_string(),
        schedule_status: true,
        event_id: 1,
        event_hose_id: 1,
        event_status: IrrigationEventStatus::InProgress.to_string(),
        event_created_at: "2021-01-01 00:00:00".to_string(),
        event_end_time: "2021-01-01 00:00:00".to_string(),
    };

    let in_the_future = Status {
        schedule_id: 1,
        schedule_days_of_week: "Monday,Tuesday,Wednesday,Thursday,Friday".to_string(),
        schedule_duration: 10,
        schedule_start_time: "2021-01-01 00:00:00".to_string(),
        schedule_status: true,
        event_id: 1,
        event_hose_id: 1,
        event_status: IrrigationEventStatus::InProgress.to_string(),
        event_created_at: "2021-01-01 00:00:00".to_string(),
        event_end_time: "2021-01-01 00:00:00".to_string(),
    };

    let now = Utc::now().naive_utc();
    let weekday = now.weekday();
    let before = now - Duration::seconds(600);
    let after = now + Duration::seconds(600);
    println!("NAIVE: {}", now);
    println!("NAIVE2: {}", before);
    println!("NAIVE3: {}", after);

    let later_today = Status {
        schedule_id: 1,
        schedule_days_of_week: "Monday,Tuesday,Wednesday,Thursday,Friday".to_string(),
        schedule_duration: 10,
        schedule_start_time: format!("{:?}", before),
        schedule_status: true,
        event_id: 1,
        event_hose_id: 1,
        event_status: IrrigationEventStatus::InProgress.to_string(),
        event_created_at: "2021-01-01 00:00:00".to_string(),
        event_end_time: "2021-01-01 00:00:00".to_string(),
    };

    let ready_to_run = Status {
        schedule_id: 1,
        schedule_days_of_week: "Monday,Tuesday,Wednesday,Thursday,Friday".to_string(),
        schedule_duration: 10,
        schedule_start_time: format!("{:?}", after),
        schedule_status: true,
        event_id: 1,
        event_hose_id: 1,
        event_status: IrrigationEventStatus::InProgress.to_string(),
        event_created_at: "2021-01-01 00:00:00".to_string(),
        event_end_time: "2021-01-01 00:00:00".to_string(),
    };

    let result = due_schedules(
        vec![in_the_past, in_the_future, later_today, ready_to_run],
        now,
    );

    println!("RESULT: {:?}", result);
}

fn test_status(
    schedule_id: i32,
    schedule_start_time: String,
    schedule_status: bool,
    schedule_days_of_week: String,
    schedule_duration: i32,
    event_id: i32,
    event_hose_id: i32,
    event_end_time: Option<String>,
) -> Status {
    // Some time in the past
    let created_at = "2021-01-01 00:00:00".to_string();
    let event_status = if event_end_time.is_some() {
        IrrigationEventStatus::Completed.to_string()
    } else if event_end_time.is_none() {
        IrrigationEventStatus::InProgress.to_string()
    } else {
        event_status.to_string()
    };

    let event_end_time = match event_end_time {
        Some(event_end_time) => event_end_time,
        None => "2021-01-01 00:00:15".to_string(),
    };

    Status {
        schedule_id,
        schedule_days_of_week,
        schedule_duration,
        schedule_start_time,
        schedule_status,
        event_id,
        event_hose_id,
        event_status,
        event_created_at: created_at,
        event_end_time,
    }
}
