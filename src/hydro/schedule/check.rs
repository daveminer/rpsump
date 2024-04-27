use anyhow::Error;
use chrono::{Datelike, NaiveDateTime, Utc};

use super::ScheduleStatus;
use crate::repository::{models::irrigation_event::IrrigationEventStatus, Repo};

pub(crate) async fn check_schedule(repo: Repo) -> Result<Vec<ScheduleStatus>, Error> {
    // Get the statuses of all the schedules
    let statuses = repo.schedule_statuses().await?;

    // Determine which statuses are due to run
    let statuses_to_run = due_statuses(statuses, Utc::now().naive_utc());
    //let status_to_run = next_due_status(statuses, Utc::now().naive_utc());

    //repo.start_irrigation_event(event_to_run).await?;

    Ok(statuses_to_run)
}

fn next_due_status(status_list: Vec<ScheduleStatus>, now: NaiveDateTime) -> ScheduleStatus {
    let mut schedules_to_run = status_list
        .into_iter()
        // Schedule is active
        .filter(|status| status.schedule.active)
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
            // If the last event was not created today
            last_event.created_at.date() != now.date()
        })
        .collect::<Vec<ScheduleStatus>>();

    schedules_to_run.sort_by(|a, b| a.schedule.start_time.cmp(&b.schedule.start_time));

    schedules_to_run[0].clone()
}

fn due_statuses(status_list: Vec<ScheduleStatus>, now: NaiveDateTime) -> Vec<ScheduleStatus> {
    let mut schedules_to_run = status_list
        .into_iter()
        // Schedule is active
        .filter(|status| status.schedule.active)
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
            // If the last event was not created today
            (last_event.created_at.date() != now.date())
                && last_event.status == IrrigationEventStatus::Queued.to_string()
        })
        .collect::<Vec<ScheduleStatus>>();

    schedules_to_run.sort_by(|a, b| a.schedule.start_time.cmp(&b.schedule.start_time));

    schedules_to_run
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDateTime;
    use rstest::rstest;

    use crate::hydro::schedule::check::due_statuses;

    use crate::hydro::schedule::ScheduleStatus;
    use crate::repository::models::irrigation_event::IrrigationEvent;
    use crate::repository::models::irrigation_schedule::IrrigationSchedule;
    use crate::test_fixtures::irrigation::schedule::{
        daily_schedule, friday_schedule, weekday_schedule,
    };
    use crate::test_fixtures::irrigation::status::no_event_today;
    use crate::test_fixtures::{
        irrigation::status::all_schedules_statuses, tests::last_friday_9pm,
    };

    #[rstest]
    fn test_due_statuses(
        all_schedules_statuses: Vec<ScheduleStatus>,
        daily_schedule: IrrigationSchedule,
        friday_schedule: IrrigationSchedule,
        last_friday_9pm: NaiveDateTime,
        weekday_schedule: IrrigationSchedule,
    ) {
        let event_ran_earlier_today = IrrigationEvent {
            id: 1,
            hose_id: 1,
            created_at: last_friday_9pm - chrono::Duration::hours(2),
            end_time: Some(
                last_friday_9pm - chrono::Duration::hours(2) + chrono::Duration::minutes(5),
            ),
            status: "Completed".to_string(),
            schedule_id: 1,
        };

        let friday_schedule = ScheduleStatus {
            schedule: friday_schedule,
            last_event: Some(event_ran_earlier_today),
        };

        let mut updated_schedule = all_schedules_statuses.clone();

        if let Some(status) = updated_schedule
            .iter_mut()
            .find(|s| s.schedule == friday_schedule.schedule)
        {
            *status = friday_schedule;
        }

        let statuses = due_statuses(updated_schedule, last_friday_9pm);

        assert_eq!(
            vec![
                no_event_today(daily_schedule),
                no_event_today(weekday_schedule)
            ],
            statuses
        );
    }
}
