use rstest::fixture;

use crate::models::{
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
        schedule_schedule_id: daily_schedule.id,
        schedule_active: daily_schedule.active,
        schedule_days_of_week: daily_schedule.days_of_week,
        schedule_duration: daily_schedule.duration,
        schedule_hoses: daily_schedule.hoses,
        schedule_name: daily_schedule.name,
        schedule_start_time: daily_schedule.start_time.to_string(),
        schedule_created_at: daily_schedule.created_at.to_string(),
        schedule_updated_at: daily_schedule.updated_at.to_string(),
        event_id: Some(completed_event.id),
        event_hose_id: Some(completed_event.hose_id),
        event_status: Some(completed_event.status),
        event_created_at: Some(completed_event.created_at.to_string()),
        event_end_time: Some(completed_event.end_time.unwrap().to_string()),
    }
}

#[fixture]
pub fn tues_thurs_schedule_status_query_result(
    tues_thurs_schedule: IrrigationSchedule,
    completed_event: IrrigationEvent,
) -> StatusQueryResult {
    StatusQueryResult {
        schedule_schedule_id: tues_thurs_schedule.id,
        schedule_active: tues_thurs_schedule.active,
        schedule_days_of_week: tues_thurs_schedule.days_of_week,
        schedule_duration: tues_thurs_schedule.duration,
        schedule_hoses: tues_thurs_schedule.hoses,
        schedule_name: tues_thurs_schedule.name,
        schedule_start_time: tues_thurs_schedule.start_time.to_string(),
        schedule_created_at: tues_thurs_schedule.created_at.to_string(),
        schedule_updated_at: tues_thurs_schedule.updated_at.to_string(),
        event_id: Some(completed_event.id),
        event_hose_id: Some(completed_event.hose_id),
        event_status: Some(completed_event.status),
        event_created_at: Some(completed_event.created_at.to_string()),
        event_end_time: Some(completed_event.end_time.unwrap().to_string()),
    }
}
