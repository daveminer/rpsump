use chrono::{Duration, NaiveDateTime, Utc};
use diesel::{
    r2d2::{ConnectionManager, PooledConnection},
    ExpressionMethods, RunQueryDsl, SqliteConnection,
};
use rpsump::{
    repository::models::{
        irrigation_event::IrrigationEventStatus, irrigation_schedule::IrrigationSchedule,
    },
    schema::irrigation_event,
};

pub async fn insert_irrigation_event(
    conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    hose: i32,
    schedule: IrrigationSchedule,
    status: IrrigationEventStatus,
) {
    let now = Utc::now().naive_utc();

    let _ = diesel::insert_into(irrigation_event::table)
        .values((
            irrigation_event::hose_id.eq(hose),
            irrigation_event::schedule_id.eq(schedule.id),
            irrigation_event::status.eq(status.to_string()),
            irrigation_event::created_at.eq(now),
        ))
        .execute(conn)
        .map_err(|e| anyhow::Error::new(e));
}

pub fn insert_completed_event(
    conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    hose: i32,
    schedule: IrrigationSchedule,
    finished_at: NaiveDateTime,
) {
    let _ = diesel::insert_into(irrigation_event::table)
        .values((
            irrigation_event::hose_id.eq(hose),
            irrigation_event::schedule_id.eq(schedule.id),
            irrigation_event::status.eq(IrrigationEventStatus::Completed.to_string()),
            irrigation_event::created_at.eq(finished_at - Duration::seconds(10)),
            irrigation_event::end_time.eq(Some(finished_at)),
        ))
        .execute(conn)
        .map_err(|e| anyhow::Error::new(e));
}
