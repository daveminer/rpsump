use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use rpsump::hydro::garden::schedule::next_scheduled_run;
use rpsump::repository::models::garden_schedule::GardenSchedule;

fn schedule(
    id: i32,
    active: bool,
    days: &str,
    start_times: &str,
    duration_secs: i32,
) -> GardenSchedule {
    GardenSchedule {
        id,
        name: format!("Schedule {}", id),
        active,
        start_times: start_times.to_string(),
        days_of_week: days.to_string(),
        duration_secs,
        created_at: NaiveDateTime::parse_from_str("2026-01-01 00:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap(),
        updated_at: NaiveDateTime::parse_from_str("2026-01-01 00:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap(),
    }
}

#[test]
fn next_scheduled_run_picks_next_future_slot_today() {
    // 2026-05-22 was a Friday.
    let now = NaiveDate::from_ymd_opt(2026, 5, 22)
        .unwrap()
        .and_time(NaiveTime::from_hms_opt(5, 30, 0).unwrap());

    let schedules = vec![schedule(1, true, "Fri", "06:00,18:00", 30)];

    let (id, when) = next_scheduled_run(&schedules, now).unwrap();
    assert_eq!(id, 1);
    assert_eq!(
        when,
        NaiveDate::from_ymd_opt(2026, 5, 22)
            .unwrap()
            .and_hms_opt(6, 0, 0)
            .unwrap()
    );
}

#[test]
fn next_scheduled_run_skips_past_times_today_and_picks_next_day() {
    let now = NaiveDate::from_ymd_opt(2026, 5, 22)
        .unwrap()
        .and_hms_opt(19, 0, 0)
        .unwrap();

    // Only Friday slot at 06:00 — should roll to next Friday.
    let schedules = vec![schedule(1, true, "Fri", "06:00", 30)];

    let (_id, when) = next_scheduled_run(&schedules, now).unwrap();
    assert_eq!(
        when,
        NaiveDate::from_ymd_opt(2026, 5, 29)
            .unwrap()
            .and_hms_opt(6, 0, 0)
            .unwrap()
    );
}

#[test]
fn next_scheduled_run_ignores_inactive() {
    let now = NaiveDate::from_ymd_opt(2026, 5, 22)
        .unwrap()
        .and_hms_opt(5, 0, 0)
        .unwrap();

    let schedules = vec![
        schedule(1, false, "Fri", "06:00", 30),
        schedule(2, true, "Sat", "07:00", 30),
    ];

    let (id, _when) = next_scheduled_run(&schedules, now).unwrap();
    assert_eq!(id, 2);
}

#[test]
fn next_scheduled_run_returns_none_when_no_active_schedules() {
    let now = NaiveDate::from_ymd_opt(2026, 5, 22)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();
    assert!(next_scheduled_run(&[], now).is_none());
}
