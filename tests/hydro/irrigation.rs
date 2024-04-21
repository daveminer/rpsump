#[cfg(test)]
mod tests {
    use chrono::{Datelike, NaiveDateTime, Utc};
    use rpsump::repository::Repo;
    use rpsump::test_fixtures::gpio::build_mock_gpio;

    use std::error::Error;
    use tokio::time::Duration;

    use crate::common::fixtures::irrigation_event::insert_irrigation_event;
    use crate::common::fixtures::irrigation_schedule::insert_irrigation_schedule;
    use crate::common::test_app::spawn_app;

    #[tokio::test]
    async fn test_irrigation_schedule() -> Result<(), Box<dyn Error>> {
        let app = spawn_app(&build_mock_gpio()).await;

        let schedules_before = app.repo.irrigation_schedules().await?;
        println!("SB: {:?}", schedules_before);
        let events_before = app.repo.irrigation_events().await?;
        println!("EB: {:?}", events_before);

        insert_test_data(app.repo).await;

        // Wait three seconds
        tokio::time::sleep(Duration::from_secs(10)).await;

        // Check that the schedules have been run
        let schedules_after = app.repo.irrigation_schedules().await?;
        println!("SA: {:?}", schedules_after);
        let events_after = app.repo.irrigation_events().await?;
        println!("EA: {:?}", events_after);

        // TODO
        // assert

        Ok(())
    }

    async fn insert_test_data(db: Repo) {
        // Get the current timestamp
        let the_past =
            NaiveDateTime::parse_from_str("2021-01-01 12:34:56".into(), "%Y-%m-%d %H:%M:%S")
                .unwrap();
        let now = Utc::now().naive_utc();
        let day = now.weekday();
        let just_passed = now - Duration::from_secs(3);
        let today_and_neighbors_str = format!(
            "{},{},{}",
            day.pred().to_string(),
            day.to_string(),
            day.succ().to_string()
        );

        // Eligible but inactive
        insert_irrigation_schedule(
            db,
            false,
            "Inactive Test Schedule".to_string(),
            just_passed.time(),
            15,
            today_and_neighbors_str.clone(),
            "1,3,4".to_string(),
            the_past,
        )
        .await;

        // Scheduled time not reached yet
        insert_irrigation_schedule(
            db,
            true,
            "Inactive Test Schedule".to_string(),
            just_passed.time(),
            15,
            day.pred().to_string(),
            "1,3,4".to_string(),
            the_past,
        )
        .await;

        // Event already ran today
        let schedule = insert_irrigation_schedule(
            db,
            true,
            "Inactive Test Schedule".to_string(),
            just_passed.time(),
            15,
            day.pred().to_string(),
            "1,3,4".to_string(),
            the_past,
        )
        .await;

        insert_irrigation_event(
            db,
            1,
            schedule.id,
            false,
            the_past,
            Some(NaiveDateTime::from(now + Duration::from_secs(1))),
        )
        .await;

        // Not scheduled today
        insert_irrigation_schedule(
            db,
            true,
            "Inactive Test Schedule".to_string(),
            just_passed.time(),
            15,
            day.pred().to_string(),
            "1,3,4".to_string(),
            the_past,
        )
        .await;

        // Eligible
        insert_irrigation_schedule(
            db,
            true,
            "Eligible Test Schedule 1".to_string(),
            just_passed.time(),
            15,
            today_and_neighbors_str,
            "1,3,4".to_string(),
            the_past,
        )
        .await;

        // Also eligible
        insert_irrigation_schedule(
            db,
            true,
            "Eligible Test Schedule 2".to_string(),
            just_passed.time(),
            15,
            "Monday,Tuesday,Wednesday,Thursday,Friday,Saturday,Sunday".to_string(),
            "2".to_string(),
            the_past,
        )
        .await;
    }
}