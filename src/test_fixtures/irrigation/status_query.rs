use rstest::fixture;

use crate::repository::models::{
    irrigation_event::{IrrigationEvent, StatusQueryResult},
    irrigation_schedule::IrrigationSchedule,
};
use crate::test_fixtures::irrigation::{
    event::completed_event, schedule::daily_schedule, schedule::tues_thurs_schedule,
};

#[fixture]
pub fn status_query_results(
    daily_schedule_status_query_result: StatusQueryResult,
    tues_thurs_schedule_status_query_result: StatusQueryResult,
) -> Vec<StatusQueryResult> {
    vec![
        daily_schedule_status_query_result.clone(),
        tues_thurs_schedule_status_query_result,
    ]
}

#[fixture]
pub fn daily_schedule_status_query_result(
    daily_schedule: IrrigationSchedule,
    completed_event: IrrigationEvent,
) -> StatusQueryResult {
    StatusQueryResult {
        id: daily_schedule.id,
        active: daily_schedule.active,
        days_of_week: daily_schedule.days_of_week,
        duration: daily_schedule.duration,
        hoses: daily_schedule.hoses,
        name: daily_schedule.name,
        start_time: daily_schedule.start_time.to_string(),
        created_at: daily_schedule.created_at.to_string(),
        updated_at: daily_schedule.updated_at.to_string(),
        event_id: Some(completed_event.id),
        hose_id: Some(completed_event.hose_id),
        status: Some(completed_event.status),
        event_created_at: Some(completed_event.created_at.to_string()),
        end_time: Some(completed_event.end_time.unwrap().to_string()),
    }
}

#[fixture]
pub fn tues_thurs_schedule_status_query_result(
    tues_thurs_schedule: IrrigationSchedule,
    completed_event: IrrigationEvent,
) -> StatusQueryResult {
    StatusQueryResult {
        id: tues_thurs_schedule.id,
        active: tues_thurs_schedule.active,
        days_of_week: tues_thurs_schedule.days_of_week,
        duration: tues_thurs_schedule.duration,
        hoses: tues_thurs_schedule.hoses,
        name: tues_thurs_schedule.name,
        start_time: tues_thurs_schedule.start_time.to_string(),
        created_at: tues_thurs_schedule.created_at.to_string(),
        updated_at: tues_thurs_schedule.updated_at.to_string(),
        event_id: Some(completed_event.id),
        hose_id: Some(completed_event.hose_id),
        status: Some(completed_event.status),
        event_created_at: Some(completed_event.created_at.to_string()),
        end_time: Some(completed_event.end_time.unwrap().to_string()),
    }
}
