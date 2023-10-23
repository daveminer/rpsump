use chrono::NaiveDateTime;
use rstest::fixture;

use crate::models::irrigation_event::{IrrigationEvent, IrrigationEventStatus};

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
