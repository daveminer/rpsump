use rstest::fixture;

use crate::hydro::schedule::Status;
use crate::repository::models::{
    irrigation_event::IrrigationEvent, irrigation_schedule::IrrigationSchedule,
};
use crate::test_fixtures::irrigation::event::completed_event;
use crate::test_fixtures::irrigation::schedule::{
    daily_schedule, deactivated_schedule, friday_schedule, tues_thurs_schedule, weekday_schedule,
    weekend_schedule,
};

#[fixture]
pub fn all_schedules_statuses(
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
            last_event: None,
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

/// Creates a `Status` instance for testing, with a completed event that is intended
/// (but does not have to be) for the same day.
///
/// Creates a daily schedule with a completed event for testing.
///
/// # Parameters
/// - `daily_schedule`: The `IrrigationSchedule` to be set in the `Status`.
/// - `completed_event`: The completed `IrrigationEvent` to be set in the `Status`.
///
/// # Returns
/// Returns a schedule `Status` instance with the given daily schedule and completed event.
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

/// Creates a `Status` instance for testing with no events today.
///
/// This function is a fixture that sets up a `Status` instance with a given
/// daily irrigation schedule and no completed irrigation event for today. It is used
/// in tests to easily create `Status` instances with specific states.
///
/// # Parameters
/// - `daily_schedule`: The daily irrigation schedule to be set in the `Status`.
///
/// # Returns
/// Returns a `Status` instance with the given daily schedule and no event for today.
#[fixture]
pub fn no_event_today(daily_schedule: IrrigationSchedule) -> Status {
    Status {
        schedule: daily_schedule,
        last_event: None,
    }
}

/// Creates a `Status` instance for testing with no scheduled events today.
///
/// This function is a fixture that sets up a `Status` instance with a given
/// daily irrigation schedule and assumes that the current day is Friday with no events scheduled.
/// It is used in tests to easily create `Status` instances with specific states.
///
/// # Parameters
/// - `daily_schedule`: The daily irrigation schedule to be set in the `Status`.
///
/// # Returns
/// Returns a `Status` instance with the given daily schedule and no scheduled event for today (Friday).
#[fixture]
pub fn not_scheduled_today(daily_schedule: IrrigationSchedule) -> Status {
    Status {
        schedule: daily_schedule,
        last_event: None,
    }
}
