pub mod run;

use anyhow::{anyhow, Error};
use chrono::{Datelike, Duration, NaiveDateTime, NaiveTime};
use diesel::RunQueryDsl;
use futures::executor::block_on;
use std::time::Duration as StdDuration;
use tokio::task::JoinHandle;
use tokio::time::sleep;

use crate::config::{MailerConfig, Settings};
use crate::controllers::spawn_blocking_with_tracing;
use crate::email::sendinblue::send_error_email;
use crate::models::irrigation_event::{IrrigationEventStatus, StatusQueryResult};
use crate::models::irrigation_schedule::IrrigationSchedule;
use crate::{database::DbPool, models::irrigation_event::IrrigationEvent};

use self::run::run_next_event;

use super::Sump;

/// Represents an IrrigationSchedule and its most recent IrrigationEvent
#[derive(Clone, Debug, PartialEq)]
pub struct Status {
    pub schedule: IrrigationSchedule,
    pub last_event: Option<IrrigationEvent>,
}

/// Intended to be run at startup and with a static lifetime. The process created
/// by this function will run on a synchronous tick. The tick will queue and
/// run events in a FIFO order.
///
///  # Arguments
///
///  * `db` - Handle to the database pool\
///  * `sump` - Instance of the Sump object for running IrrigationEvents
///
pub fn start(db: DbPool, sump: Sump) -> JoinHandle<()> {
    // Schedule runs in a new thread
    spawn_blocking_with_tracing(move || {
        loop {
            let thread_sump = sump.clone();
            let thread_db = db.clone();
            // Synchronously check for schedules to run
            block_on(sleep(StdDuration::from_secs(5)));

            // Get the statuses of all the schedules
            let statuses = match get_schedule_statuses(thread_db.clone()) {
                Ok(statuses) => statuses,
                Err(e) => {
                    tracing::error!("Could not get schedule statuses: {}", e);

                    continue;
                }
            };

            // Determine which statuses are due to run
            let events_to_insert = due_statuses(statuses, chrono::Utc::now().naive_utc());

            // Insert a queued event for each due status
            events_to_insert.into_iter().for_each(|status| {
                if let Err(e) = block_on(IrrigationEvent::create_irrigation_events_for_status(
                    thread_db.clone(),
                    status,
                )) {
                    tracing::error!("Could not insert irrigation event: {}", e)
                }
            });

            // Run irrigation events
            run_next_event(thread_db, thread_sump);
        }
    })
}

fn get_schedule_statuses(db: DbPool) -> Result<Vec<Status>, Error> {
    let mut conn = match db.get() {
        Ok(conn) => conn,
        Err(e) => return Err(e.into()),
    };

    IrrigationEvent::status_query()
        .load::<StatusQueryResult>(&mut conn)
        .map(|results| {
            let statuses = results
                .into_iter()
                .map(|result: StatusQueryResult| {
                    let StatusQueryResult {
                        schedule_schedule_id,
                        schedule_active,
                        schedule_name,
                        schedule_duration,
                        schedule_start_time,
                        schedule_days_of_week,
                        schedule_hoses,
                        schedule_created_at,
                        schedule_updated_at,
                        event_id,
                        event_hose_id,
                        event_status,
                        event_created_at,
                        event_end_time,
                        ..
                    } = result;

                    let schedule = IrrigationSchedule {
                        id: schedule_schedule_id,
                        active: schedule_active,
                        name: schedule_name,
                        duration: schedule_duration,
                        start_time: NaiveTime::parse_from_str(&schedule_start_time, "%H:%M:%S")
                            .unwrap(),
                        days_of_week: schedule_days_of_week,
                        hoses: schedule_hoses,
                        created_at: NaiveDateTime::parse_from_str(
                            &schedule_created_at,
                            "%Y-%m-%d %H:%M:%S",
                        )
                        .unwrap(),
                        updated_at: NaiveDateTime::parse_from_str(
                            &schedule_updated_at,
                            "%Y-%m-%d %H:%M:%S",
                        )
                        .unwrap(),
                    };

                    if event_id.is_none() {
                        return Status {
                            schedule,
                            last_event: None,
                        };
                    }

                    let last_event = IrrigationEvent {
                        id: event_id.unwrap(),
                        hose_id: event_hose_id.unwrap(),
                        schedule_id: schedule_schedule_id,
                        status: event_status.unwrap(),
                        end_time: Some(
                            NaiveDateTime::parse_from_str(
                                &event_end_time.unwrap(),
                                "%Y-%m-%d %H:%M:%S",
                            )
                            .unwrap(),
                        ),
                        created_at: NaiveDateTime::parse_from_str(
                            &event_created_at.unwrap(),
                            "%Y-%m-%d %H:%M:%S",
                        )
                        .unwrap(),
                    };

                    Status {
                        schedule,
                        last_event: Some(last_event),
                    }
                })
                .collect::<Vec<Status>>();

            statuses
        })
        .map_err(|e| anyhow!(e))
}

fn is_schedule_running(status_list: Vec<Status>) -> Result<bool, Error> {
    let active_schedules = status_list
        .into_iter()
        .filter(|status| {
            if let Some(event) = &status.last_event {
                event.status == IrrigationEventStatus::InProgress.to_string()
            } else {
                false
            }
        })
        .collect::<Vec<Status>>();

    if active_schedules.is_empty() {
        return Ok(false);
    };

    let active_schedule = if active_schedules.len() > 1 {
        return Err(anyhow!("Found more than one event running at one time."));
    } else {
        active_schedules[0].clone()
    };

    let last_event = match active_schedule.last_event {
        Some(last_event) => last_event,
        None => return Ok(false),
    };

    let now = chrono::Utc::now().naive_utc();
    if now - last_event.created_at > Duration::seconds(60) {
        // TODO: Stop event; send error email, create dynamic max runtime variable
    }

    return Ok(true);
}

fn due_statuses(status_list: Vec<Status>, now: NaiveDateTime) -> Vec<Status> {
    let mut schedules_to_run = status_list
        .into_iter()
        // Schedule is active
        .filter(|status| status.schedule.active == true)
        // Schedule is for today
        .filter(|status| {
            status
                .schedule
                .days_of_week
                .contains(&now.weekday().to_string())
        })
        // Schedule's run time has passed
        .filter(|status| status.schedule.start_time < now.time())
        // Schedule has not been queued already today
        .filter(|status| {
            if status.last_event.is_none() {
                return true;
            }

            let last_event = status.last_event.clone().unwrap();
            last_event.created_at.date() != now.date()
        })
        .collect::<Vec<Status>>();

    schedules_to_run.sort_by(|a, b| a.schedule.start_time.cmp(&b.schedule.start_time));

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

#[cfg(test)]
mod tests {
    use chrono::NaiveDateTime;
    use rstest::*;

    use crate::{
        sump::schedule::{due_statuses, Status},
        test_fixtures::{irrigation::all_schedules_statuses, test_time},
    };

    // "Test Schedule Friday" has an event that has run already; the others
    // are deactivated or not run on Fridays. This leaves two schedules due.
    #[rstest]
    fn due_statuses_success(
        #[from(all_schedules_statuses)] statuses: Vec<Status>,
        #[from(test_time)] time: NaiveDateTime,
    ) {
        let due = due_statuses(statuses, time);

        assert!(due.len() == 2);
        assert!(due
            .iter()
            .find(|s| s.schedule.name == "Test Schedule Daily")
            .is_some());
        assert!(due
            .iter()
            .find(|s| s.schedule.name == "Test Schedule Weekday")
            .is_some());
    }

    // #[rstest]
    // fn due_statuses_run_needed_success(
    //     #[from(no_event_today)] status: Status,
    //     #[from(test_time)] time: NaiveDateTime,
    // ) {
    //     println!("TIME: {}", time.clone().weekday());
    //     assert!(due_statuses(vec![status.clone()], time).first().unwrap() == &status);
    // }
}
