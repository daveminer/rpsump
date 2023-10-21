use chrono::{Duration, Utc};
use rpsump::{models::irrigation_event::IrrigationEventStatus, sump::schedule::Status};
use rstest::fixture;

#[fixture]
pub fn finished_status(
    #[default(1)] schedule_id: i32,
    #[default("Monday,Tuesday,Wednesday,Thursday,Friday,Saturday,Sunday")]
    schedule_days_of_week: String,
    #[default("12:00:00")] schedule_start_time: String,
    #[default(1)] event_id: i32,
    #[default(1)] event_hose_id: i32,
) -> Status {
    let now = Utc::now().naive_utc();
    let event_created_at = now - Duration::days(365);

    Status {
        schedule_id,
        schedule_days_of_week,
        schedule_duration: 15,
        schedule_start_time,
        schedule_status: true,
        event_id,
        event_hose_id,
        event_status: IrrigationEventStatus::Completed.to_string(),
        event_created_at: event_created_at.to_string(),
        event_end_time: (event_created_at + Duration::seconds(schedule_duration.into()))
            .to_string(),
    }
}

#[fixture]
pub fn ready_status(
    #[default(1)] schedule_id: i32,
    #[default("Monday,Tuesday,Wednesday,Thursday,Friday,Saturday,Sunday")]
    schedule_days_of_week: String,
    #[default("12:00:00")] schedule_start_time: String,
    #[default(true)] schedule_status: bool,
    #[default(1)] event_id: i32,
    #[default(1)] event_hose_id: i32,
) -> Status {
    let now = Utc::now().naive_utc();

    Status {
        schedule_id,
        schedule_days_of_week,
        schedule_duration,
        schedule_start_time,
        schedule_status,
        event_id,
        event_hose_id,
        event_status: IrrigationEventStatus::InProgress.to_string(),
        event_created_at: now.to_string(),
        event_end_time: "".to_string(),
    }
}
