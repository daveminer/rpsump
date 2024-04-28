#[cfg(test)]
mod tests {
    use chrono::{Datelike, NaiveDateTime, Utc, Weekday};
    use rpsump::repository::models::irrigation_event::IrrigationEventStatus;
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

        insert_test_data(app.repo).await;

        let events_before = app.repo.irrigation_events().await?;
        let queued_events = events_before
            .iter()
            .filter(|e| e.status == IrrigationEventStatus::Queued.to_string())
            .count();

        let in_prog_events = events_before
            .iter()
            .filter(|e| e.status == IrrigationEventStatus::InProgress.to_string())
            .count();

        let completed_events = events_before
            .iter()
            .filter(|e| e.status == IrrigationEventStatus::Completed.to_string())
            .count();

        assert!(queued_events == 1);
        assert!(in_prog_events == 0);
        assert!(completed_events == 0);

        // Wait long enough for two 1-second jobs to complete
        tokio::time::sleep(Duration::from_secs(5)).await;

        // Check that the schedules have been run
        let events_after = app.repo.irrigation_events().await?;
        let queued_events = events_after
            .iter()
            .filter(|e| e.status == IrrigationEventStatus::Queued.to_string())
            .count();

        let in_prog_events = events_after
            .iter()
            .filter(|e| e.status == IrrigationEventStatus::InProgress.to_string())
            .count();

        let completed_events = events_after
            .iter()
            .filter(|e| e.status == IrrigationEventStatus::Completed.to_string())
            .count();

        assert!(queued_events == 3);
        assert!(in_prog_events == 0);
        assert!(completed_events == 2);

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

        let active_days = vec![day.pred(), day, day.succ()];

        // Eligible but inactive
        insert_irrigation_schedule(
            db,
            false,
            "Inactive Test Schedule".to_string(),
            just_passed.time(),
            1,
            active_days.clone(),
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
            1,
            vec![day.pred()],
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
            1,
            vec![day.pred()],
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
            1,
            vec![day.pred()],
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
            1,
            active_days,
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
            1,
            vec![
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri,
                Weekday::Sat,
                Weekday::Sun,
            ],
            "2".to_string(),
            the_past,
        )
        .await;
    }
}
