//! Precipitation-aware skip decisions for garden schedules.
//!
//! The skip logic is intentionally a pure function over a [`PrecipSnapshot`].
//! HTTP fetching lives in [`WeatherClient`] and produces snapshots; tests
//! construct snapshots directly without going through HTTP.
//!
//! Decision:
//!   skip = available
//!       AND schedule.skip_on_rain
//!       AND (current_mm >= active_rain_mm
//!            OR past_mm + forecast_weight * forecast_mm >= threshold_mm)
//!
//! If the snapshot is `available == false` (API down or feature disabled),
//! we never skip — better a wasted watering than a parched bed.

use std::sync::Arc;

use anyhow::{anyhow, Error};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::repository::models::garden_schedule::GardenSchedule;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WeatherConfig {
    pub enabled: bool,
    pub latitude: f32,
    pub longitude: f32,
    pub past_hours: u32,
    pub forecast_hours: u32,
    pub threshold_mm: f32,
    pub forecast_weight: f32,
    pub active_rain_mm: f32,
    pub cache_ttl_secs: u64,
}

impl WeatherConfig {
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            latitude: 0.0,
            longitude: 0.0,
            past_hours: 24,
            forecast_hours: 12,
            threshold_mm: 5.0,
            forecast_weight: 0.5,
            active_rain_mm: 0.3,
            cache_ttl_secs: 900,
        }
    }
}

/// A point-in-time view of precipitation that is enough to make a skip
/// decision without further I/O. Constructed by `WeatherClient::snapshot`
/// or directly in tests.
#[derive(Clone, Copy, Debug)]
pub struct PrecipSnapshot {
    pub past_mm: f32,
    pub forecast_mm: f32,
    pub current_mm: f32,
    pub threshold_mm: f32,
    pub forecast_weight: f32,
    pub active_rain_mm: f32,
    pub available: bool,
}

impl PrecipSnapshot {
    pub fn unavailable() -> Self {
        Self {
            past_mm: 0.0,
            forecast_mm: 0.0,
            current_mm: 0.0,
            threshold_mm: 0.0,
            forecast_weight: 0.0,
            active_rain_mm: 0.0,
            available: false,
        }
    }

    pub fn effective_mm(&self) -> f32 {
        self.past_mm + self.forecast_weight * self.forecast_mm
    }

    pub fn is_actively_raining(&self) -> bool {
        self.current_mm >= self.active_rain_mm
    }

    pub fn is_wet(&self) -> bool {
        if !self.available {
            return false;
        }
        self.is_actively_raining() || self.effective_mm() >= self.threshold_mm
    }
}

/// Pure: should this specific schedule be skipped right now, given a snapshot?
/// Honors the per-schedule `skip_on_rain` flag so covered/indoor zones can opt out.
pub fn should_skip(schedule: &GardenSchedule, precip: &PrecipSnapshot) -> bool {
    schedule.skip_on_rain && precip.is_wet()
}

// --- HTTP client ----------------------------------------------------------

#[derive(Clone, Debug)]
struct CachedReading {
    fetched_at: DateTime<Utc>,
    past_mm: f32,
    forecast_mm: f32,
    current_mm: f32,
}

#[derive(Clone)]
pub struct WeatherClient {
    cfg: WeatherConfig,
    http: reqwest::Client,
    cache: Arc<RwLock<Option<CachedReading>>>,
}

#[derive(Deserialize)]
struct OpenMeteoResponse {
    hourly: OpenMeteoHourly,
}

#[derive(Deserialize)]
struct OpenMeteoHourly {
    precipitation: Vec<f32>,
}

impl WeatherClient {
    pub fn new(cfg: WeatherConfig) -> Self {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("reqwest client");
        Self {
            cfg,
            http,
            cache: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn snapshot(&self) -> PrecipSnapshot {
        if !self.cfg.enabled {
            return PrecipSnapshot::unavailable();
        }
        match self.reading().await {
            Ok(r) => PrecipSnapshot {
                past_mm: r.past_mm,
                forecast_mm: r.forecast_mm,
                current_mm: r.current_mm,
                threshold_mm: self.cfg.threshold_mm,
                forecast_weight: self.cfg.forecast_weight,
                active_rain_mm: self.cfg.active_rain_mm,
                available: true,
            },
            Err(e) => {
                tracing::warn!("weather lookup failed, watering anyway: {e}");
                PrecipSnapshot::unavailable()
            }
        }
    }

    async fn reading(&self) -> Result<CachedReading, Error> {
        let ttl = Duration::seconds(self.cfg.cache_ttl_secs as i64);
        if let Some(r) = self.cache.read().await.clone() {
            if Utc::now() - r.fetched_at < ttl {
                return Ok(r);
            }
        }
        let r = self.fetch().await?;
        *self.cache.write().await = Some(r.clone());
        Ok(r)
    }

    async fn fetch(&self) -> Result<CachedReading, Error> {
        let url = format!(
            "https://api.open-meteo.com/v1/forecast\
             ?latitude={lat}&longitude={lon}\
             &hourly=precipitation\
             &past_hours={past}&forecast_hours={fwd}\
             &timezone=UTC",
            lat = self.cfg.latitude,
            lon = self.cfg.longitude,
            past = self.cfg.past_hours,
            fwd = self.cfg.forecast_hours,
        );

        let resp: OpenMeteoResponse = self.http.get(&url).send().await?.json().await?;
        let past_n = self.cfg.past_hours as usize;
        let values = resp.hourly.precipitation;

        if values.len() < past_n + 1 {
            return Err(anyhow!(
                "open-meteo returned {} hours, expected at least {}",
                values.len(),
                past_n + 1
            ));
        }

        // Layout (confirmed against api.open-meteo.com): [past..., current, forecast...]
        // index 0..past_n      = past_hours observed/analyzed hours
        // index past_n         = current (in-progress) hour
        // index past_n+1..end  = forecast hours after the current hour
        let past_mm: f32 = values[..past_n].iter().sum();
        let current_mm: f32 = values[past_n];
        let forecast_mm: f32 = values[past_n + 1..].iter().sum();

        Ok(CachedReading {
            fetched_at: Utc::now(),
            past_mm,
            forecast_mm,
            current_mm,
        })
    }
}

// --- tests ---------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;

    fn schedule(id: i32, skip_on_rain: bool) -> GardenSchedule {
        GardenSchedule {
            id,
            name: format!("zone {id}"),
            active: true,
            start_times: "06:00".to_string(),
            days_of_week: "Mon".to_string(),
            duration_secs: 60,
            skip_on_rain,
            created_at: NaiveDateTime::default(),
            updated_at: NaiveDateTime::default(),
        }
    }

    fn snap(past: f32, current: f32, forecast: f32, available: bool) -> PrecipSnapshot {
        PrecipSnapshot {
            past_mm: past,
            current_mm: current,
            forecast_mm: forecast,
            threshold_mm: 5.0,
            forecast_weight: 0.5,
            active_rain_mm: 0.3,
            available,
        }
    }

    #[test]
    fn effective_mm_adds_weighted_forecast() {
        let s = snap(2.0, 0.0, 4.0, true);
        // 2.0 + 0.5 * 4.0 = 4.0
        assert!((s.effective_mm() - 4.0).abs() < 1e-6);
    }

    #[test]
    fn dry_snapshot_is_not_wet() {
        assert!(!snap(0.0, 0.0, 0.0, true).is_wet());
    }

    #[test]
    fn unavailable_snapshot_is_never_wet() {
        // Big numbers but unavailable → still not wet.
        let s = snap(100.0, 100.0, 100.0, false);
        assert!(!s.is_wet());
    }

    #[test]
    fn active_rain_short_circuits_threshold() {
        // current_mm above active_rain_mm but effective_mm well under threshold.
        let s = snap(0.0, 0.5, 0.0, true);
        assert!(s.is_actively_raining());
        assert!(s.effective_mm() < s.threshold_mm);
        assert!(s.is_wet());
    }

    #[test]
    fn drizzle_below_active_rain_floor_does_not_trip_active_check() {
        let s = snap(0.0, 0.1, 0.0, true);
        assert!(!s.is_actively_raining());
        assert!(!s.is_wet());
    }

    #[test]
    fn past_alone_can_meet_threshold() {
        let s = snap(8.0, 0.0, 0.0, true);
        assert!(s.is_wet());
    }

    #[test]
    fn forecast_alone_must_exceed_threshold_after_weighting() {
        // 9mm forecast * 0.5 weight = 4.5, under 5.0 threshold.
        let under = snap(0.0, 0.0, 9.0, true);
        assert!(!under.is_wet());

        // 12mm forecast * 0.5 weight = 6.0, over threshold.
        let over = snap(0.0, 0.0, 12.0, true);
        assert!(over.is_wet());
    }

    #[test]
    fn threshold_boundary_is_inclusive() {
        let s = snap(5.0, 0.0, 0.0, true);
        assert!(s.is_wet());
    }

    #[test]
    fn should_skip_dry_runs_everything() {
        let s = snap(0.0, 0.0, 0.0, true);
        assert!(!should_skip(&schedule(1, true), &s));
        assert!(!should_skip(&schedule(2, false), &s));
    }

    #[test]
    fn should_skip_wet_skips_only_rain_aware_schedules() {
        let s = snap(8.0, 0.0, 0.0, true);
        assert!(should_skip(&schedule(1, true), &s));
        assert!(!should_skip(&schedule(2, false), &s));
    }

    #[test]
    fn should_skip_unavailable_never_skips() {
        let s = snap(100.0, 100.0, 100.0, false);
        assert!(!should_skip(&schedule(1, true), &s));
    }

    #[test]
    fn should_skip_wet_but_schedule_opts_out_runs() {
        let s = snap(20.0, 5.0, 20.0, true);
        assert!(!should_skip(&schedule(42, false), &s));
    }
}
