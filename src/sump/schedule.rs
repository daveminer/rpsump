use anyhow::{anyhow, Error};
use chrono::{Datelike, Duration, NaiveDateTime};
use diesel::prelude::*;
use diesel::sql_types::{Bool, Integer, Text};
use diesel::RunQueryDsl;
use futures::executor::block_on;
use rppal::gpio::Level;
use std::time::{Duration as StdDuration, SystemTime};
use tokio::task::JoinHandle;
use tokio::time::sleep;

use crate::config::{MailerConfig, Settings};
use crate::controllers::spawn_blocking_with_tracing;
use crate::email::sendinblue::send_error_email;
use crate::models::irrigation_event::IrrigationEventStatus;
use crate::{database::DbPool, models::irrigation_event::IrrigationEvent};

use super::{SharedOutputPin, Sump};

static INVALID_HOSE_NUMBER_MSG: &str = "Invalid hose number provided";

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
    //let db_clone = db.clone();
    let mailer_config = settings.mailer.clone();

    spawn_blocking_with_tracing(move || {
        //let sump_clone = sump.clone();
        loop {
            block_on(sleep(StdDuration::from_secs(5)));

            let statuses = match schedule_status(db.clone()) {
                Ok(statuses) => statuses,
                Err(e) => {
                    error_email(&mailer_config, e);

                    continue;
                }
            };

            match schedule_tick(statuses) {
                Ok(statuses) => {
                    if statuses.len() == 0 {
                        continue;
                    }

                    // Run a schedule
                    schedule_run(db.clone(), statuses, sump.clone());
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

fn schedule_run(db: DbPool, statuses: Vec<Status>, sump: Sump) {
    let status = statuses[0].clone();
    // Create an event for the first schedule in the list
    if let Err(e) = IrrigationEvent::create_irrigation_event(db.clone(), status.clone()) {
        tracing::error!("Error creating irrigation event: {:?}", e);
        return;
    }

    // Start the irrigation I/O in a new thread
    let _irrigation_job = start_irrigation(db, status, sump);
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

fn start_irrigation(db: DbPool, status: Status, sump: Sump) -> JoinHandle<Result<(), Error>> {
    let sump = sump.clone();
    let status = status.clone();
    tokio::spawn(async move {
        let sensor_state = match sump.sensor_state.lock() {
            Ok(sensor_state) => *sensor_state,
            Err(e) => {
                tracing::error!("Could not get sensor state: {}", e);
                // TODO: send email
                //return Ok(ApiResponse::internal_server_error());
                return Err(anyhow!(e.to_string()));
            }
        };
        if sensor_state.irrigation_low_sensor == Level::Low {
            // Exit if water is too low
            tracing::warn!("Water is too low to start irrigation.");
            ()
        }

        let hose_pin = match choose_irrigation_valve_pin(status.event_hose_id, sump.clone()) {
            Ok(pin) => pin,
            Err(e) => {
                tracing::error!("Invalid pin from schedule");
                return Err(e);
            }
        };

        // Open the solenoid for the job
        let mut hose = hose_pin.lock().unwrap();
        // Start the pump
        hose.set_high();
        drop(hose);

        let start_time = SystemTime::now();
        // Start the pump
        let mut pump = match sump.irrigation_pump_control_pin.lock() {
            Ok(pump) => pump,
            Err(e) => {
                tracing::error!("Could not get sump pump control pin: {}", e);
                return Err(anyhow!(e.to_string()));
            }
        };

        pump.set_high();
        drop(pump);

        // Wait for the job to finish
        if status.schedule_duration > 60 {
            tracing::error!("Schedule duration is too long");
            return Err(anyhow!("Schedule duration is too long"));
        }

        while !job_complete(status.schedule_duration, start_time) {
            block_on(sleep(StdDuration::from_secs(1)));
        }

        // Stop the pump
        let mut pump = match sump.irrigation_pump_control_pin.lock() {
            Ok(pump) => pump,
            Err(e) => {
                tracing::error!("Could not get sump pump control pin: {}", e);
                return Err(anyhow!(e.to_string()));
            }
        };
        pump.set_low();

        // Close the solenoid
        let mut hose = hose_pin.lock().unwrap();
        // Stop the pump
        hose.set_low();

        // Move the job out of "in progress" status
        let _ = block_on(IrrigationEvent::finish(db));
        return Ok(());
    })
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

fn choose_irrigation_valve_pin(hose_id: i32, sump: Sump) -> Result<SharedOutputPin, Error> {
    if hose_id == 1 {
        Ok(sump.irrigation_valve_1_control_pin)
    } else if hose_id == 2 {
        Ok(sump.irrigation_valve_2_control_pin)
    } else if hose_id == 3 {
        Ok(sump.irrigation_valve_3_control_pin)
    } else if hose_id == 4 {
        Ok(sump.irrigation_valve_4_control_pin)
    } else {
        tracing::error!(INVALID_HOSE_NUMBER_MSG);
        Err(anyhow!(INVALID_HOSE_NUMBER_MSG))
    }
}

fn job_complete(duration: i32, start_time: SystemTime) -> bool {
    let elapsed = match SystemTime::now().duration_since(start_time) {
        Ok(now) => now,
        Err(e) => {
            tracing::error!("Error getting duration since start time: {:?}", e);
            return true;
        }
    };

    let dur = match duration.try_into() {
        Ok(dur) => dur,
        Err(e) => {
            tracing::error!("Error converting duration to std duration: {:?}", e);
            return true;
        }
    };

    elapsed >= StdDuration::from_secs(dur)
}

#[tokio::test]
async fn schedules_due_ready() {
    let status = Status {
        schedule_id: 1,
        schedule_days_of_week: "1,2,3,4,5,6,7".to_string(),
        schedule_duration: 10,
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
