use chrono::{NaiveDateTime, NaiveTime};
use rstest::fixture;

use crate::{
    models::{
        irrigation_event::{IrrigationEvent, IrrigationEventStatus},
        irrigation_schedule::IrrigationSchedule,
    },
    sump::schedule::Status,
};

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

#[fixture]
pub fn daily_schedule(
    #[default(1)] id: i32,
    #[default("Test Schedule Daily")] name: String,
    #[default(15)] duration: i32,
    #[default("12:00:00")] start_time: NaiveTime,
    #[default("Monday,Tuesday,Wednesday,Thursday,Friday,Saturday,Sunday")] days_of_week: String,
    #[default("1,2,3,4")] hoses: String,
    #[default(true)] active: bool,
    #[default(NaiveDateTime::parse_from_str("2021-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap())]
    created_at: NaiveDateTime,
    #[default(NaiveDateTime::parse_from_str("2021-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap())]
    updated_at: NaiveDateTime,
) -> IrrigationSchedule {
    IrrigationSchedule {
        id,
        name,
        duration,
        start_time,
        days_of_week,
        hoses,
        active,
        created_at,
        updated_at,
    }
}

#[fixture]
pub fn completed_event(
    #[default(1)] id: i32,
    #[default(1)] hose_id: i32,
    #[default(NaiveDateTime::parse_from_str("2021-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap())]
    created_at: NaiveDateTime,
    #[default(Some(NaiveDateTime::parse_from_str("2021-01-01 12:00:15", "%Y-%m-%d %H:%M:%S").unwrap()))]
    end_time: Option<NaiveDateTime>,
    #[default(IrrigationEventStatus::Completed)] status: IrrigationEventStatus,
    #[default(1)] schedule_id: i32,
) -> IrrigationEvent {
    IrrigationEvent {
        id,
        hose_id,
        created_at,
        end_time,
        status: status.to_string(),
        schedule_id,
    }
}
