use chrono::NaiveDateTime;
use diesel::{ExpressionMethods, RunQueryDsl};
use rpsump::repository::{
    models::{irrigation_event::IrrigationEventStatus, irrigation_schedule::IrrigationSchedule},
    Repo,
};

pub async fn insert_irrigation_event(
    repo: Repo,
    hose: i32,
    schedule: i32,
    event_status: String,
    event_created_at: NaiveDateTime,
    event_end_time: Option<NaiveDateTime>,
) {
    let schedule = IrrigationSchedule {
        id: schedule,
        name: "Test Schedule".into(),
        active: event_status.parse().unwrap(),
        created_at: event_created_at,
        updated_at: event_created_at,
        duration: (event_end_time.unwrap() - event_created_at).num_seconds() as i32,
        start_time: event_created_at.time(),
        days_of_week: "Monday".into(),
        hoses: hose.to_string(),
    };
    repo.create_irrigation_event(schedule, hose).await.unwrap();
}

pub async fn insert_irrigation_events(repo: Repo) {
    let complete_status: String = IrrigationEventStatus::Completed.to_string();

    let dt =
        NaiveDateTime::parse_from_str("2022-01-01 12:34:56".into(), "%Y-%m-%d %H:%M:%S").unwrap();
    insert_irrigation_event(
        repo,
        1,
        1,
        complete_status.clone(),
        dt,
        Some(dt + chrono::Duration::seconds(30)),
    )
    .await;
    let dt2 =
        NaiveDateTime::parse_from_str("2022-01-02 16:50:22".into(), "%Y-%m-%d %H:%M:%S").unwrap();
    insert_irrigation_event(
        repo,
        1,
        1,
        complete_status.clone(),
        dt2,
        Some(dt2 + chrono::Duration::seconds(30)),
    )
    .await;
    let dt3 =
        NaiveDateTime::parse_from_str("2022-01-03 23:59:59".into(), "%Y-%m-%d %H:%M:%S").unwrap();
    insert_irrigation_event(
        repo,
        1,
        1,
        complete_status.clone(),
        dt3,
        Some(dt3 + chrono::Duration::seconds(30)),
    )
    .await;
    let dt4 =
        NaiveDateTime::parse_from_str("2022-01-04 02:10:08".into(), "%Y-%m-%d %H:%M:%S").unwrap();
    insert_irrigation_event(
        repo,
        1,
        1,
        complete_status.clone(),
        dt4,
        Some(dt4 + chrono::Duration::seconds(30)),
    )
    .await;
    let dt5 =
        NaiveDateTime::parse_from_str("2022-01-05 12:34:56".into(), "%Y-%m-%d %H:%M:%S").unwrap();
    insert_irrigation_event(
        repo,
        1,
        1,
        complete_status,
        dt5,
        Some(dt5 + chrono::Duration::seconds(30)),
    )
    .await;
}
