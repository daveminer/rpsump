//! Aggregates garden history into the rollups the Reports screen shows.
//!
//! The aggregation is a pure function over the events in a window so it can be
//! unit tested without a database. A window is small enough (a couple of weeks
//! of a single-zone drip line) to roll up in memory rather than in SQL.

use actix_web::{
    get,
    web::{Data, Query},
    HttpResponse, Result,
};
use chrono::{Duration, NaiveDate, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::auth::authenticated_user::AuthenticatedUser;
use crate::controllers::auth::helpers::error_response;
use crate::repository::models::garden_event::{
    GardenEvent, GardenEventFilter, GardenEventSource, GardenEventStatus,
};
use crate::repository::Repo;

pub const DEFAULT_REPORT_DAYS: i64 = 14;
pub const MAX_REPORT_DAYS: i64 = 90;
const MAX_RECENT_RAIN_SKIPS: usize = 10;

#[derive(Debug, Deserialize)]
pub struct ReportQuery {
    pub days: Option<i64>,
}

#[derive(Debug, Default, PartialEq, Serialize)]
pub struct RunTotals {
    pub runs: i64,
    pub seconds: i64,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct DayBucket {
    pub date: NaiveDate,
    pub seconds: i64,
    pub runs: i64,
    pub rain_skips: i64,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct ScheduleBreakdown {
    pub schedule_id: Option<i32>,
    pub name: Option<String>,
    /// True once the schedule itself is gone; its history is kept by name.
    pub deleted: bool,
    pub runs: i64,
    pub seconds: i64,
    pub last_run_at: Option<NaiveDateTime>,
    pub rain_skips: i64,
    pub rain_skip_seconds: i64,
    pub last_rain_skip_at: Option<NaiveDateTime>,
}

#[derive(Debug, Default, PartialEq, Serialize)]
pub struct Outcomes {
    pub completed: i64,
    pub in_progress: i64,
    pub queued: i64,
    pub cancelled: i64,
    pub skipped: i64,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct RainSkip {
    pub event_id: i32,
    pub schedule_id: Option<i32>,
    pub name: Option<String>,
    pub at: NaiveDateTime,
    pub duration_secs: i32,
}

#[derive(Debug, Default, PartialEq, Serialize)]
pub struct RainSkipSummary {
    pub runs: i64,
    pub seconds_avoided: i64,
    pub recent: Vec<RainSkip>,
}

#[derive(Debug, Default, PartialEq, Serialize)]
pub struct SourceTotals {
    pub scheduled: RunTotals,
    pub manual: RunTotals,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct GardenReport {
    pub from: NaiveDateTime,
    pub to: NaiveDateTime,
    pub days: i64,
    pub daily: Vec<DayBucket>,
    pub totals: SourceTotals,
    pub by_schedule: Vec<ScheduleBreakdown>,
    pub outcomes: Outcomes,
    pub rain_skips: RainSkipSummary,
}

/// Seconds of water delivered by events at or after `from`.
pub fn watered_secs_since(
    events: &[GardenEvent],
    from: NaiveDateTime,
    now: NaiveDateTime,
) -> i64 {
    events
        .iter()
        .filter(|e| e.occurred_at() >= from)
        .map(|e| e.watered_secs(now))
        .sum()
}

/// Number of runs the rain skipped at or after `from`.
pub fn rain_skips_since(events: &[GardenEvent], from: NaiveDateTime) -> i64 {
    events
        .iter()
        .filter(|e| e.occurred_at() >= from)
        .filter(|e| e.parsed_status() == Some(GardenEventStatus::Skipped))
        .count() as i64
}

/// Groups a schedule's history. Events keep their schedule's name even after
/// the schedule is deleted (which nulls the FK), so a deleted schedule stays a
/// group of its own rather than collapsing in with every other deleted one.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum GroupKey {
    Id(i32),
    Name(String),
    Unknown,
}

fn group_key(event: &GardenEvent) -> GroupKey {
    match (event.schedule_id, event.schedule_name.as_ref()) {
        (Some(id), _) => GroupKey::Id(id),
        (None, Some(name)) => GroupKey::Name(name.clone()),
        (None, None) => GroupKey::Unknown,
    }
}

/// The name each group's most recent event carried.
fn latest_names(events: &[&GardenEvent]) -> HashMap<GroupKey, String> {
    let mut newest: HashMap<GroupKey, (NaiveDateTime, String)> = HashMap::new();

    for event in events {
        let Some(name) = event.schedule_name.clone() else {
            continue;
        };
        let at = event.occurred_at();
        newest
            .entry(group_key(event))
            .and_modify(|(seen, current)| {
                if at > *seen {
                    *seen = at;
                    *current = name.clone();
                }
            })
            .or_insert((at, name));
    }

    newest
        .into_iter()
        .map(|(key, (_, name))| (key, name))
        .collect()
}

/// Builds the report from every event whose `occurred_at` falls in
/// `[from, to]`. `events` may be in any order.
pub fn build_report(
    events: &[GardenEvent],
    from: NaiveDateTime,
    to: NaiveDateTime,
    now: NaiveDateTime,
) -> GardenReport {
    let in_window: Vec<&GardenEvent> = events
        .iter()
        .filter(|e| {
            let at = e.occurred_at();
            at >= from && at <= to
        })
        .collect();

    let days = (to.date() - from.date()).num_days() + 1;

    // One bucket per calendar day, including the days nothing ran.
    let mut buckets: Vec<DayBucket> = (0..days)
        .map(|offset| DayBucket {
            date: from.date() + Duration::days(offset),
            seconds: 0,
            runs: 0,
            rain_skips: 0,
        })
        .collect();

    let mut totals = SourceTotals::default();
    let mut outcomes = Outcomes::default();
    let mut rain_skips = RainSkipSummary::default();
    let mut groups: HashMap<GroupKey, ScheduleBreakdown> = HashMap::new();

    // A rename leaves older events carrying the old name, so label each group
    // with the name its most recent event was queued under.
    let latest_names = latest_names(&in_window);

    for event in &in_window {
        let status = event.parsed_status();
        let watered = event.watered_secs(now);
        let started = event.start_time.is_some();
        let skipped = status == Some(GardenEventStatus::Skipped);
        let at = event.occurred_at();

        match status {
            Some(GardenEventStatus::Completed) => outcomes.completed += 1,
            Some(GardenEventStatus::InProgress) => outcomes.in_progress += 1,
            Some(GardenEventStatus::Queued) => outcomes.queued += 1,
            Some(GardenEventStatus::Cancelled) => outcomes.cancelled += 1,
            Some(GardenEventStatus::Skipped) => outcomes.skipped += 1,
            None => (),
        }

        if let Some(bucket) = buckets.iter_mut().find(|b| b.date == at.date()) {
            bucket.seconds += watered;
            if started {
                bucket.runs += 1;
            }
            if skipped {
                bucket.rain_skips += 1;
            }
        }

        let totals_entry = match event.parsed_source() {
            Some(GardenEventSource::Manual) => &mut totals.manual,
            // An event with an unreadable source is still a run that used
            // water; counting it as scheduled beats dropping it.
            _ => &mut totals.scheduled,
        };
        totals_entry.seconds += watered;
        if started {
            totals_entry.runs += 1;
        }

        if skipped {
            rain_skips.runs += 1;
            rain_skips.seconds_avoided += event.skipped_secs();
            rain_skips.recent.push(RainSkip {
                event_id: event.id,
                schedule_id: event.schedule_id,
                name: event.schedule_name.clone(),
                at,
                duration_secs: event.duration_secs,
            });
        }

        // Manual runs have no schedule to break down by; they are reported in
        // `totals.manual` instead.
        if event.parsed_source() == Some(GardenEventSource::Manual) {
            continue;
        }

        let key = group_key(event);
        let entry = groups.entry(key.clone()).or_insert_with(|| ScheduleBreakdown {
            schedule_id: event.schedule_id,
            name: latest_names.get(&key).cloned(),
            deleted: event.schedule_id.is_none(),
            runs: 0,
            seconds: 0,
            last_run_at: None,
            rain_skips: 0,
            rain_skip_seconds: 0,
            last_rain_skip_at: None,
        });

        entry.seconds += watered;
        if started {
            entry.runs += 1;
            if entry.last_run_at.map_or(true, |last| at > last) {
                entry.last_run_at = Some(at);
            }
        }
        if skipped {
            entry.rain_skips += 1;
            entry.rain_skip_seconds += event.skipped_secs();
            if entry.last_rain_skip_at.map_or(true, |last| at > last) {
                entry.last_rain_skip_at = Some(at);
            }
        }
    }

    rain_skips.recent.sort_by(|a, b| b.at.cmp(&a.at));
    rain_skips.recent.truncate(MAX_RECENT_RAIN_SKIPS);

    let mut by_schedule: Vec<ScheduleBreakdown> = groups.into_values().collect();
    by_schedule.sort_by(|a, b| {
        a.name
            .as_deref()
            .unwrap_or("")
            .cmp(b.name.as_deref().unwrap_or(""))
            .then(a.schedule_id.cmp(&b.schedule_id))
    });

    GardenReport {
        from,
        to,
        days,
        daily: buckets,
        totals,
        by_schedule,
        outcomes,
        rain_skips,
    }
}

#[get("/report")]
#[tracing::instrument(skip(repo, _user))]
pub async fn report(
    query: Query<ReportQuery>,
    repo: Data<Repo>,
    _user: AuthenticatedUser,
) -> Result<HttpResponse> {
    let days = query
        .days
        .unwrap_or(DEFAULT_REPORT_DAYS)
        .clamp(1, MAX_REPORT_DAYS);

    let from = (Utc::now().naive_utc().date() - Duration::days(days - 1))
        .and_hms_opt(0, 0, 0)
        .expect("midnight is always a valid time");

    let events = match repo.garden_events(GardenEventFilter::since(from)).await {
        Ok(events) => events,
        Err(e) => return Ok(error_response(e, "Could not get garden events")),
    };

    // Read the clock after the query so an event that starts while it runs
    // still lands inside the window.
    let now = Utc::now().naive_utc();

    Ok(HttpResponse::Ok().json(build_report(&events, from, now, now)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dt(s: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").unwrap()
    }

    fn event(
        id: i32,
        schedule: Option<(i32, &str)>,
        source: GardenEventSource,
        status: GardenEventStatus,
        scheduled_for: &str,
        duration_secs: i32,
        run: Option<(&str, &str)>,
    ) -> GardenEvent {
        GardenEvent {
            id,
            schedule_id: schedule.map(|(id, _)| id),
            schedule_name: schedule.map(|(_, name)| name.to_string()),
            source: source.to_string(),
            status: status.to_string(),
            scheduled_for: dt(scheduled_for),
            duration_secs,
            start_time: run.map(|(start, _)| dt(start)),
            end_time: run.map(|(_, end)| dt(end)),
            created_at: dt(scheduled_for),
        }
    }

    /// Mirrors the Reports screen: three scheduled runs, one manual run and
    /// two rain skips across a fortnight.
    fn fixture() -> Vec<GardenEvent> {
        vec![
            event(
                1,
                Some((1, "Morning drip")),
                GardenEventSource::Scheduled,
                GardenEventStatus::Completed,
                "2026-08-26 13:11:00",
                1800,
                Some(("2026-08-26 13:11:00", "2026-08-26 13:41:00")),
            ),
            event(
                2,
                Some((1, "Morning drip")),
                GardenEventSource::Scheduled,
                GardenEventStatus::Skipped,
                "2026-08-27 15:24:00",
                180,
                None,
            ),
            event(
                3,
                Some((1, "Morning drip")),
                GardenEventSource::Scheduled,
                GardenEventStatus::Completed,
                "2026-08-28 15:18:00",
                180,
                Some(("2026-08-28 15:18:00", "2026-08-28 15:21:00")),
            ),
            event(
                4,
                Some((2, "Evening top-up")),
                GardenEventSource::Scheduled,
                GardenEventStatus::Skipped,
                "2026-08-28 16:24:00",
                120,
                None,
            ),
            event(
                5,
                None,
                GardenEventSource::Manual,
                GardenEventStatus::Completed,
                "2026-08-29 15:11:00",
                120,
                Some(("2026-08-29 15:11:00", "2026-08-29 15:13:00")),
            ),
            event(
                6,
                Some((1, "Morning drip")),
                GardenEventSource::Scheduled,
                GardenEventStatus::Completed,
                "2026-08-29 16:24:00",
                180,
                Some(("2026-08-29 16:24:00", "2026-08-29 16:27:00")),
            ),
        ]
    }

    fn report_for(events: &[GardenEvent]) -> GardenReport {
        let now = dt("2026-08-29 18:00:00");
        build_report(events, dt("2026-08-16 00:00:00"), now, now)
    }

    #[test]
    fn splits_totals_by_source() {
        let summary = report_for(&fixture());

        assert_eq!(
            summary.totals.scheduled,
            RunTotals {
                runs: 3,
                seconds: 1800 + 180 + 180
            }
        );
        assert_eq!(
            summary.totals.manual,
            RunTotals {
                runs: 1,
                seconds: 120
            }
        );
    }

    #[test]
    fn counts_every_outcome() {
        let summary = report_for(&fixture());

        assert_eq!(
            summary.outcomes,
            Outcomes {
                completed: 4,
                in_progress: 0,
                queued: 0,
                cancelled: 0,
                skipped: 2,
            }
        );
    }

    #[test]
    fn buckets_one_day_per_date_including_empty_days() {
        let summary = report_for(&fixture());

        assert_eq!(summary.days, 14);
        assert_eq!(summary.daily.len(), 14);
        assert_eq!(summary.daily[0].date, dt("2026-08-16 00:00:00").date());
        assert_eq!(summary.daily[0].seconds, 0);

        let aug_26 = summary
            .daily
            .iter()
            .find(|b| b.date == dt("2026-08-26 00:00:00").date())
            .unwrap();
        assert_eq!(aug_26.seconds, 1800);
        assert_eq!(aug_26.runs, 1);

        let aug_27 = summary
            .daily
            .iter()
            .find(|b| b.date == dt("2026-08-27 00:00:00").date())
            .unwrap();
        assert_eq!(aug_27.seconds, 0);
        assert_eq!(aug_27.rain_skips, 1);

        // The manual run and the scheduled run share Aug 29.
        let aug_29 = summary
            .daily
            .iter()
            .find(|b| b.date == dt("2026-08-29 00:00:00").date())
            .unwrap();
        assert_eq!(aug_29.seconds, 300);
        assert_eq!(aug_29.runs, 2);
    }

    #[test]
    fn breaks_down_by_schedule_and_leaves_manual_runs_out() {
        let summary = report_for(&fixture());

        assert_eq!(summary.by_schedule.len(), 2);

        let evening = &summary.by_schedule[0];
        assert_eq!(evening.name.as_deref(), Some("Evening top-up"));
        assert_eq!(evening.runs, 0);
        assert_eq!(evening.seconds, 0);
        assert_eq!(evening.rain_skips, 1);
        assert_eq!(evening.last_rain_skip_at, Some(dt("2026-08-28 16:24:00")));

        let morning = &summary.by_schedule[1];
        assert_eq!(morning.name.as_deref(), Some("Morning drip"));
        assert_eq!(morning.runs, 3);
        assert_eq!(morning.seconds, 2160);
        assert_eq!(morning.last_run_at, Some(dt("2026-08-29 16:24:00")));
        assert_eq!(morning.rain_skips, 1);
        assert!(!morning.deleted);
    }

    #[test]
    fn summarizes_rain_skips_newest_first() {
        let summary = report_for(&fixture());

        assert_eq!(summary.rain_skips.runs, 2);
        assert_eq!(summary.rain_skips.seconds_avoided, 300);
        assert_eq!(summary.rain_skips.recent.len(), 2);
        assert_eq!(summary.rain_skips.recent[0].at, dt("2026-08-28 16:24:00"));
        assert_eq!(
            summary.rain_skips.recent[0].name.as_deref(),
            Some("Evening top-up")
        );
        assert_eq!(summary.rain_skips.recent[1].at, dt("2026-08-27 15:24:00"));
    }

    #[test]
    fn keeps_a_deleted_schedules_history_under_its_own_name() {
        let mut events = fixture();
        // Deleting a schedule nulls the FK but leaves the name behind.
        for event in events.iter_mut() {
            if event.schedule_id == Some(2) {
                event.schedule_id = None;
            }
        }
        events.push(event(
            7,
            None,
            GardenEventSource::Scheduled,
            GardenEventStatus::Completed,
            "2026-08-25 12:00:00",
            60,
            Some(("2026-08-25 12:00:00", "2026-08-25 12:01:00")),
        ));

        let summary = report_for(&events);

        // Evening top-up (deleted, by name), Morning drip, and the nameless
        // orphan stay three separate groups.
        assert_eq!(summary.by_schedule.len(), 3);

        let unnamed = summary
            .by_schedule
            .iter()
            .find(|s| s.name.is_none())
            .unwrap();
        assert_eq!(unnamed.runs, 1);
        assert!(unnamed.deleted);

        let evening = summary
            .by_schedule
            .iter()
            .find(|s| s.name.as_deref() == Some("Evening top-up"))
            .unwrap();
        assert!(evening.deleted);
        assert_eq!(evening.rain_skips, 1);
    }

    #[test]
    fn a_run_stopped_early_counts_only_what_it_used() {
        let events = vec![event(
            1,
            Some((1, "Morning drip")),
            GardenEventSource::Scheduled,
            GardenEventStatus::Cancelled,
            "2026-08-29 12:00:00",
            1800,
            Some(("2026-08-29 12:00:00", "2026-08-29 12:00:30")),
        )];

        let summary = report_for(&events);

        assert_eq!(summary.totals.scheduled.seconds, 30);
        assert_eq!(summary.outcomes.cancelled, 1);
    }

    #[test]
    fn a_run_cancelled_before_it_started_counts_nothing() {
        let events = vec![event(
            1,
            None,
            GardenEventSource::Manual,
            GardenEventStatus::Cancelled,
            "2026-08-29 12:00:00",
            1800,
            None,
        )];

        let summary = report_for(&events);

        assert_eq!(summary.totals.manual, RunTotals::default());
    }

    #[test]
    fn excludes_events_outside_the_window() {
        let summary = build_report(
            &fixture(),
            dt("2026-08-28 00:00:00"),
            dt("2026-08-29 18:00:00"),
            dt("2026-08-29 18:00:00"),
        );

        assert_eq!(summary.days, 2);
        assert_eq!(summary.daily.len(), 2);
        assert_eq!(summary.outcomes.completed, 3);
        assert_eq!(summary.totals.scheduled.seconds, 360);
    }

    #[test]
    fn window_helpers_measure_from_a_rolling_start() {
        let events = fixture();
        let now = dt("2026-08-29 18:00:00");

        assert_eq!(
            watered_secs_since(&events, now - Duration::hours(24), now),
            300
        );
        assert_eq!(
            watered_secs_since(&events, now - Duration::days(3), now),
            480
        );
        assert_eq!(
            watered_secs_since(&events, now - Duration::days(7), now),
            2280
        );
        // Only the Aug 28 skip falls inside 48 hours of Aug 29 18:00.
        assert_eq!(rain_skips_since(&events, now - Duration::hours(48)), 1);
        assert_eq!(rain_skips_since(&events, now - Duration::hours(72)), 2);
    }
}
