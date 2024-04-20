pub mod gpio;
pub mod irrigation;
pub mod settings;

#[cfg(test)]
pub mod tests {
    use chrono::{Datelike, NaiveDateTime, Utc};
    use rstest::fixture;

    #[fixture]
    pub fn time() -> NaiveDateTime {
        NaiveDateTime::parse_from_str("2021-01-01 13:00:00", "%Y-%m-%d %H:%M:%S").unwrap()
    }

    #[fixture]
    pub fn last_friday_9pm() -> NaiveDateTime {
        let now = Utc::now();
        let today = now.date_naive();
        let weekday = today.weekday();

        let days_since_last_friday = (weekday.num_days_from_monday() + 3) % 7;
        let last_friday = today - chrono::Duration::days(days_since_last_friday as i64);

        last_friday.and_hms_opt(21, 0, 0).unwrap()
    }
}
