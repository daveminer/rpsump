use rstest::fixture;

use crate::models::{irrigation_event::IrrigationEvent, irrigation_schedule::IrrigationSchedule};
use crate::sump::schedule::Status;
use crate::test_fixtures::irrigation::event::completed_event;
use crate::test_fixtures::irrigation::schedule::{
    daily_schedule, deactivated_schedule, friday_schedule, tues_thurs_schedule, weekday_schedule,
    weekend_schedule,
};

#[fixture]
pub fn all_schedules_statuses(
    completed_event: IrrigationEvent,
    daily_schedule: IrrigationSchedule,
    friday_schedule: IrrigationSchedule,
    tues_thurs_schedule: IrrigationSchedule,
    weekday_schedule: IrrigationSchedule,
    weekend_schedule: IrrigationSchedule,
    deactivated_schedule: IrrigationSchedule,
) -> Vec<Status> {
    vec![
        Status {
            schedule: daily_schedule,
            last_event: None,
        },
        Status {
            schedule: friday_schedule,
            last_event: Some(completed_event),
        },
        Status {
            schedule: tues_thurs_schedule,
            last_event: None,
        },
        Status {
            schedule: weekday_schedule,
            last_event: None,
        },
        Status {
            schedule: weekend_schedule,
            last_event: None,
        },
        Status {
            schedule: deactivated_schedule,
            last_event: None,
        },
    ]
}

#[fixture]
pub fn event_finished_today(
    daily_schedule: IrrigationSchedule,
    completed_event: IrrigationEvent,
) -> Status {
    Status {
        schedule: daily_schedule,
        last_event: Some(completed_event),
    }
}

#[fixture]
pub fn no_event_today(daily_schedule: IrrigationSchedule) -> Status {
    Status {
        schedule: daily_schedule,
        last_event: None,
    }
}

// Assumes that the current day is Friday
#[fixture]
pub fn not_scheduled_today(daily_schedule: IrrigationSchedule) -> Status {
    Status {
        schedule: daily_schedule,
        last_event: None,
    }
}
