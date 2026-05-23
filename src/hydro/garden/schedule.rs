use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, Utc, Weekday};
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

use crate::hydro::garden::Garden;
use crate::hydro::weather::WeatherClient;
use crate::repository::models::{
    garden_event::{GardenEvent, GardenEventStatus},
    garden_schedule::GardenSchedule,
};
use crate::repository::Repo;

/// Spawned at startup. Each tick: queue any newly-due events, then drive the
/// next queued event to completion before sleeping again.
pub fn start(
    repo: Repo,
    garden: Garden,
    weather: WeatherClient,
    frequency_sec: u64,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            let now = Utc::now().naive_utc();
            let precip = weather.snapshot().await;

            if let Err(e) = repo.queue_due_garden_events(now, &precip).await {
                tracing::error!("garden: failed to queue due events: {}", e);
            }

            match repo.next_queued_garden_event().await {
                Ok(Some(event)) => {
                    if let Err(e) = run_event(repo, &garden, event).await {
                        tracing::error!("garden: run_event failed: {}", e);
                    }
                }
                Ok(None) => {}
                Err(e) => tracing::error!("garden: failed to fetch next event: {}", e),
            }

            sleep(Duration::from_secs(frequency_sec)).await;
        }
    })
}

pub async fn run_event(
    repo: Repo,
    garden: &Garden,
    event: GardenEvent,
) -> anyhow::Result<()> {
    let event_id = event.id;
    repo.begin_garden_event(event_id).await?;

    let duration_secs = clamp_duration(event.duration_secs, garden.max_seconds_runtime);
    tracing::info!(event_id, duration_secs, "garden: opening solenoid");

    {
        let mut lock = garden.solenoid.pin.lock().await;
        lock.on();
    }

    let mut terminal_status = GardenEventStatus::Completed;
    let mut elapsed = 0i32;
    while elapsed < duration_secs {
        sleep(Duration::from_secs(1)).await;
        elapsed += 1;

        // Check for external cancellation (e.g. POST /garden/stop)
        match repo.garden_event_by_id(event_id).await {
            Ok(Some(current)) => {
                if current.status == GardenEventStatus::Cancelled.to_string() {
                    terminal_status = GardenEventStatus::Cancelled;
                    break;
                }
            }
            Ok(None) => {
                tracing::warn!(event_id, "garden: event vanished mid-run, stopping");
                terminal_status = GardenEventStatus::Cancelled;
                break;
            }
            Err(e) => tracing::warn!("garden: failed to poll event status: {}", e),
        }
    }

    {
        let mut lock = garden.solenoid.pin.lock().await;
        lock.off();
    }
    tracing::info!(event_id, ?terminal_status, "garden: closed solenoid");

    repo.finish_garden_event(event_id, terminal_status).await?;
    Ok(())
}

fn clamp_duration(requested: i32, max: u32) -> i32 {
    let max = max as i32;
    if requested <= 0 {
        0
    } else if requested > max {
        max
    } else {
        requested
    }
}

/// Returns the next future `(schedule, scheduled_for)` across all active
/// schedules, walking up to 7 days forward. Used by the `/garden/status`
/// endpoint to surface when the system will next run.
pub fn next_scheduled_run(
    schedules: &[GardenSchedule],
    now: NaiveDateTime,
) -> Option<(i32, NaiveDateTime)> {
    let mut best: Option<(i32, NaiveDateTime)> = None;
    for schedule in schedules.iter().filter(|s| s.active) {
        let days = schedule.parsed_days_of_week();
        let times = schedule.parsed_start_times();
        if days.is_empty() || times.is_empty() {
            continue;
        }
        for offset in 0..=7 {
            let candidate_date = now.date() + chrono::Duration::days(offset);
            let weekday = candidate_date.weekday();
            if !days.iter().any(|d| *d == weekday) {
                continue;
            }
            for time in &times {
                let candidate = candidate_date.and_time(*time);
                if candidate <= now {
                    continue;
                }
                match best {
                    None => best = Some((schedule.id, candidate)),
                    Some((_, current)) if candidate < current => {
                        best = Some((schedule.id, candidate));
                    }
                    _ => {}
                }
            }
        }
    }
    best
}

#[allow(dead_code)]
fn weekday_matches(days: &[Weekday], date: NaiveDate) -> bool {
    days.iter().any(|d| *d == date.weekday())
}

#[allow(dead_code)]
fn at(date: NaiveDate, time: NaiveTime) -> NaiveDateTime {
    date.and_time(time)
}
