use chrono::{NaiveDateTime, NaiveTime, Weekday};
use rpsump::repository::{
    models::irrigation_schedule::{CreateIrrigationScheduleParams, IrrigationSchedule},
    Repo,
};

pub async fn insert_irrigation_schedule(
    repo: Repo,
    sched_active: bool,
    sched_name: String,
    sched_start_time: NaiveTime,
    sched_duration: i32,
    sched_days: Vec<Weekday>,
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
        days_of_week: sched_days,
        hoses: hose_vec,
    };

    repo.create_irrigation_schedule(schedule).await.unwrap()
}

pub async fn insert_irrigation_schedules_fixed(repo: Repo, count: u8) {
    let now =
        NaiveDateTime::parse_from_str("2021-01-01 12:34:56".into(), "%Y-%m-%d %H:%M:%S").unwrap();

    let dt1 = NaiveTime::parse_from_str("12:34:56".into(), "%H:%M:%S").unwrap();
    insert_irrigation_schedule(
        repo,
        true,
        "Schedule 1".to_string(),
        dt1,
        15,
        vec![Weekday::Mon, Weekday::Wed],
        //"Monday,Wednesday".into(),
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
        15,
        vec![Weekday::Tue, Weekday::Fri],
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
        true,
        "Schedule 3".to_string(),
        dt3,
        15,
        vec![
            Weekday::Mon,
            Weekday::Tue,
            Weekday::Wed,
            Weekday::Thu,
            Weekday::Fri,
        ],
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
        15,
        vec![Weekday::Mon],
        "1".into(),
        now,
    )
    .await;
}
