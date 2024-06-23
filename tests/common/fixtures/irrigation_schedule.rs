use chrono::{Datelike, NaiveDateTime, NaiveTime, Utc, Weekday};
use diesel::{result::Error, Connection, ExpressionMethods, RunQueryDsl};
use rpsump::{
    repository::{
        models::irrigation_schedule::{CreateIrrigationScheduleParams, IrrigationSchedule},
        Repo,
    },
    schema::irrigation_schedule,
};
use tokio::time::Duration;

use rpsump::schema::irrigation_schedule as irrigation_schedule_dsl;

use super::irrigation_event::insert_completed_event;

pub async fn insert_eligible_schedule_first_run(repo: Repo) -> IrrigationSchedule {
    let now = Utc::now().naive_utc();

    let schedule = CreateIrrigationScheduleParams {
        active: true,
        name: "Active Schedule First Run".into(),
        start_time: now.time() - Duration::from_secs(5),
        duration: 1,
        days_of_week: vec![now.weekday()],
        hoses: vec![1, 2],
    };

    repo.create_irrigation_schedule(schedule).await.unwrap()
}

pub async fn insert_eligible_schedule_subsequent_run(repo: Repo) -> IrrigationSchedule {
    let now = Utc::now().naive_utc();
    let finished_at = now - Duration::from_secs(60 * 60 * 24);

    let mut conn = repo.pool().await.unwrap().get().unwrap();
    let result = conn
        .transaction(|conn| {
            let schedule = diesel::insert_into(irrigation_schedule::table)
                .values((
                    irrigation_schedule_dsl::active.eq(true),
                    irrigation_schedule_dsl::name.eq("Eligible Subsequent Run"),
                    irrigation_schedule_dsl::duration.eq(1),
                    irrigation_schedule_dsl::start_time.eq(now.time() - Duration::from_secs(10)),
                    irrigation_schedule_dsl::days_of_week.eq(format!("{:?}", vec![now.weekday()])),
                    irrigation_schedule_dsl::hoses.eq("[1]"),
                ))
                .get_result::<IrrigationSchedule>(conn)
                .unwrap();

            insert_completed_event(conn, 1, schedule.clone(), finished_at);

            Ok::<IrrigationSchedule, Error>(schedule)
        })
        .unwrap();

    result
}

pub async fn insert_inactive_schedule(repo: Repo) -> IrrigationSchedule {
    let now = Utc::now().naive_utc();

    let schedule = CreateIrrigationScheduleParams {
        active: false,
        name: "Inactive Schedule".into(),
        start_time: now.time() - Duration::from_secs(5),
        duration: 1,
        days_of_week: vec![now.weekday()],
        hoses: vec![1],
    };

    repo.create_irrigation_schedule(schedule).await.unwrap()
}

pub async fn insert_finished_schedule(repo: Repo) -> IrrigationSchedule {
    let now = Utc::now().naive_utc();
    let finished_at = now - Duration::from_secs(60);

    let mut conn = repo.pool().await.unwrap().get().unwrap();
    let result = conn
        .transaction(|conn| {
            let schedule = diesel::insert_into(irrigation_schedule::table)
                .values((
                    irrigation_schedule_dsl::active.eq(true),
                    irrigation_schedule_dsl::name.eq("Finished Schedule"),
                    irrigation_schedule_dsl::duration.eq(1),
                    irrigation_schedule_dsl::start_time.eq(now.time() - Duration::from_secs(10)),
                    irrigation_schedule_dsl::days_of_week.eq(format!("{:?}", vec![now.weekday()])),
                    irrigation_schedule_dsl::hoses.eq("[3,4]"),
                ))
                .get_result::<IrrigationSchedule>(conn)
                .unwrap();

            insert_completed_event(
                conn,
                3,
                schedule.clone(),
                finished_at - Duration::from_secs(20),
            );

            insert_completed_event(
                conn,
                4,
                schedule.clone(),
                finished_at - Duration::from_secs(30),
            );

            Ok::<IrrigationSchedule, Error>(schedule)
        })
        .unwrap();

    result
}

pub async fn insert_not_today_schedule(repo: Repo) -> IrrigationSchedule {
    let now = Utc::now().naive_utc();
    let finished_at = now - Duration::from_secs(60 * 60 * 24);

    let mut conn = repo.pool().await.unwrap().get().unwrap();
    let result = conn
        .transaction(|conn| {
            let schedule = diesel::insert_into(irrigation_schedule::table)
                .values((
                    irrigation_schedule_dsl::active.eq(true),
                    irrigation_schedule_dsl::name.eq("Finished Schedule"),
                    irrigation_schedule_dsl::duration.eq(1),
                    irrigation_schedule_dsl::start_time.eq(now.time() - Duration::from_secs(10)),
                    irrigation_schedule_dsl::days_of_week.eq(format!("{:?}", vec![now.weekday()])),
                    irrigation_schedule_dsl::hoses.eq("[3,4]"),
                ))
                .get_result::<IrrigationSchedule>(conn)
                .unwrap();

            insert_completed_event(conn, 3, schedule.clone(), finished_at);
            insert_completed_event(
                conn,
                4,
                schedule.clone(),
                finished_at + Duration::from_secs(10),
            );

            Ok::<IrrigationSchedule, Error>(schedule)
        })
        .unwrap();

    result
}

pub async fn insert_pending_schedule(repo: Repo) -> IrrigationSchedule {
    let now = Utc::now().naive_utc();
    let day = now.weekday();
    let time = now.time();

    let schedule = CreateIrrigationScheduleParams {
        active: true,
        name: "Pending".into(),
        start_time: time - Duration::from_secs(3),
        duration: 1,
        days_of_week: vec![day.pred(), now.weekday(), day.succ()],
        hoses: vec![4],
    };

    repo.create_irrigation_schedule(schedule).await.unwrap()
}

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
