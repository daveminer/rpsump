use chrono::{NaiveDateTime, NaiveTime};
use diesel::{ExpressionMethods, RunQueryDsl};
use rpsump::{
    database::DbPool,
    schema::{irrigation_schedule, irrigation_schedule::*},
};

pub async fn insert_irrigation_schedule(
    db: DbPool,
    sched_active: bool,
    sched_name: String,
    sched_start_time: NaiveTime,
    sched_duration: i32,
    sched_days: String,
    sched_hoses: String,
    sched_created_at: NaiveDateTime,
) {
    let mut conn = db.get().unwrap();
    diesel::insert_into(irrigation_schedule::table)
        .values((
            active.eq(sched_active),
            name.eq(sched_name),
            start_time.eq(sched_start_time.to_string()),
            duration.eq(sched_duration),
            days_of_week.eq(sched_days),
            hoses.eq(sched_hoses),
            created_at.eq(sched_created_at),
            updated_at.eq(sched_created_at),
        ))
        .execute(&mut conn)
        .unwrap();
}

pub async fn insert_irrigation_schedules(db: DbPool, count: u8) {
    let now =
        NaiveDateTime::parse_from_str("2021-01-01 12:34:56".into(), "%Y-%m-%d %H:%M:%S").unwrap();

    let dt1 = NaiveTime::parse_from_str("12:34:56".into(), "%H:%M:%S").unwrap();
    insert_irrigation_schedule(
        db.clone(),
        true,
        "Schedule 1".to_string(),
        dt1,
        15,
        "Monday,Wednesday".into(),
        "3".into(),
        now,
    )
    .await;

    if count < 2 {
        return;
    }
    let dt2 = NaiveTime::parse_from_str("12:34:56".into(), "%H:%M:%S").unwrap();
    insert_irrigation_schedule(
        db.clone(),
        true,
        "Schedule 2".to_string(),
        dt2,
        15,
        "Tuesday,Friday".into(),
        "1,2,3,4".into(),
        now,
    )
    .await;

    if count < 3 {
        return;
    }
    let dt3 = NaiveTime::parse_from_str("12:34:56".into(), "%H:%M:%S").unwrap();
    insert_irrigation_schedule(
        db.clone(),
        true,
        "Schedule 3".to_string(),
        dt3,
        15,
        "Monday,Tuesday,Wednesday,Thursday,Friday".into(),
        "2,4".into(),
        now,
    )
    .await;

    if count < 4 {
        return;
    }
    let dt4 = NaiveTime::parse_from_str("12:34:56".into(), "%H:%M:%S").unwrap();
    insert_irrigation_schedule(
        db.clone(),
        false,
        "Schedule 4".to_string(),
        dt4,
        15,
        "Monday".into(),
        "1".into(),
        now,
    )
    .await;
}
