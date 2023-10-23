use chrono::NaiveDateTime;
use rstest::fixture;

pub mod irrigation;

#[fixture]
pub fn test_time() -> NaiveDateTime {
    NaiveDateTime::parse_from_str("2021-01-01 13:00:00", "%Y-%m-%d %H:%M:%S").unwrap()
}
