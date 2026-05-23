use serde_json::{json, Value};

pub fn schedule_params() -> Value {
    json!({
        "name": "Test Schedule",
        "active": true,
        "start_times": ["06:00", "18:30"],
        "days_of_week": ["Mon", "Wed", "Fri"],
        "duration_secs": 30,
    })
}

pub fn schedule_params_named(name: &str) -> Value {
    json!({
        "name": name,
        "active": true,
        "start_times": ["06:00"],
        "days_of_week": ["Mon"],
        "duration_secs": 30,
    })
}
