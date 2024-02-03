use chrono::{NaiveDateTime, NaiveTime};
use rstest::fixture;

use crate::repository::models::irrigation_schedule::IrrigationSchedule;

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
