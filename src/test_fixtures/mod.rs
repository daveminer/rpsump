pub mod gpio;
pub mod hydro;
pub mod irrigation;

#[cfg(test)]
pub mod tests {
    use chrono::NaiveDateTime;
    use rstest::fixture;

    #[fixture]
    pub fn time() -> NaiveDateTime {
        NaiveDateTime::parse_from_str("2021-01-01 13:00:00", "%Y-%m-%d %H:%M:%S").unwrap()
    }
}
