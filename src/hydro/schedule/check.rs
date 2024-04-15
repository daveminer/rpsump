use anyhow::Error;
use chrono::{Datelike, NaiveDateTime};

use super::Status;
use crate::repository::Repo;

pub(crate) async fn check_schedule(repo: Repo) -> Result<(), Error> {
    // Get the statuses of all the schedules
    let statuses = repo.schedule_statuses().await?;

    // Determine which statuses are due to run
    let events_to_insert = due_statuses(statuses, chrono::Utc::now().naive_utc());

    let _rows_inserted = repo.queue_irrigation_events(events_to_insert).await?;
    println!("Checking Done");
    Ok(())
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

    println!("Schedules to run: {:?}", schedules_to_run);
    return schedules_to_run;
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDateTime;
    use rstest::rstest;

    use crate::hydro::schedule::check::due_statuses;

    use crate::hydro::schedule::Status;
    use crate::repository::models::irrigation_schedule::IrrigationSchedule;
    use crate::test_fixtures::irrigation::schedule::{
        daily_schedule, friday_schedule, weekday_schedule,
    };
    use crate::test_fixtures::irrigation::status::{event_finished_today, no_event_today};
    use crate::test_fixtures::{
        irrigation::status::all_schedules_statuses, tests::last_friday_9pm,
    };

    #[rstest]
    fn test_due_statuses(
        all_schedules_statuses: Vec<Status>,
        daily_schedule: IrrigationSchedule,
        friday_schedule: IrrigationSchedule,
        last_friday_9pm: NaiveDateTime,
        weekday_schedule: IrrigationSchedule,
    ) {
        let statuses = due_statuses(all_schedules_statuses.clone(), last_friday_9pm);

        assert_eq!(
            vec![
                no_event_today(daily_schedule),
                event_finished_today(
                    friday_schedule,
                    all_schedules_statuses[1].last_event.clone().unwrap()
                ),
                no_event_today(weekday_schedule)
            ],
            statuses
        );
    }
}
