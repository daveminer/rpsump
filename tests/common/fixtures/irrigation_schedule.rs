use chrono::{NaiveDateTime, NaiveTime};
use rpsump::repository::{
    models::irrigation_schedule::{CreateIrrigationScheduleParams, DayOfWeek, IrrigationSchedule},
    Repo,
};

pub async fn insert_irrigation_schedule(
    repo: Repo,
    sched_active: bool,
    sched_name: String,
    sched_start_time: NaiveTime,
    sched_duration: i32,
    #[allow(unused)] sched_days: String,
    sched_hoses: String,
    #[allow(unused)] sched_created_at: NaiveDateTime,
) -> IrrigationSchedule {
    let hose_vec: Vec<i32> = sched_hoses
        .clone()
        .split(',')
        .map(|s| s.parse().unwrap())
        .collect();

    let schedule = CreateIrrigationScheduleParams {
        active: sched_active,
        name: sched_name.clone(),
        start_time: sched_start_time,
        duration: sched_duration,
        days_of_week: vec![DayOfWeek::Monday, DayOfWeek::Tuesday],
        // TODO: create serde for this
        hoses: hose_vec,
    };

    repo.create_irrigation_schedule(schedule).await.unwrap()
}

pub async fn insert_irrigation_schedules(repo: Repo, count: u8) {
    let now =
        NaiveDateTime::parse_from_str("2021-01-01 12:34:56".into(), "%Y-%m-%d %H:%M:%S").unwrap();

    let dt1 = NaiveTime::parse_from_str("12:34:56".into(), "%H:%M:%S").unwrap();
    insert_irrigation_schedule(
        repo,
        false,
        "Schedule 1".to_string(),
        dt1,
        1,
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
        repo,
        true,
        "Schedule 2".to_string(),
        dt2,
        1,
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
        repo,
        false,
        "Schedule 3".to_string(),
        dt3,
        1,
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
        repo,
        false,
        "Schedule 4".to_string(),
        dt4,
        1,
        "Monday".into(),
        "1".into(),
        now,
    )
    .await;
}
