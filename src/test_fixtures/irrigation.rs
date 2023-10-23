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
pub fn friday_schedule(
    #[default(1)] id: i32,
    #[default("Test Schedule Friday")] name: String,
    #[default(15)] duration: i32,
    #[default("12:00:00")] start_time: NaiveTime,
    #[default("Friday")] days_of_week: String,
    #[default("2,3")] hoses: String,
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
pub fn tues_thurs_schedule(
    #[default(1)] id: i32,
    #[default("Test Schedule Tues Thurs")] name: String,
    #[default(15)] duration: i32,
    #[default("12:00:00")] start_time: NaiveTime,
    #[default("Tuesday,Thursday")] days_of_week: String,
    #[default("1,2,3")] hoses: String,
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
pub fn weekday_schedule(
    #[default(1)] id: i32,
    #[default("Test Schedule Weekday")] name: String,
    #[default(15)] duration: i32,
    #[default("12:00:00")] start_time: NaiveTime,
    #[default("Monday,Tuesday,Wednesday,Thursday,Friday")] days_of_week: String,
    #[default("2,3,4")] hoses: String,
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
pub fn weekend_schedule(
    #[default(1)] id: i32,
    #[default("Test Schedule Weekend")] name: String,
    #[default(15)] duration: i32,
    #[default("12:00:00")] start_time: NaiveTime,
    #[default("Saturday,Sunday")] days_of_week: String,
    #[default("1,2,3")] hoses: String,
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
pub fn deactivated_schedule(
    #[default(1)] id: i32,
    #[default("Test Schedule Deactivated")] name: String,
    #[default(15)] duration: i32,
    #[default("12:00:00")] start_time: NaiveTime,
    #[default("Monday,Tuesday,Wednesday,Thursday,Friday,Saturday,Sunday")] days_of_week: String,
    #[default("1,2,3")] hoses: String,
    #[default(false)] active: bool,
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
