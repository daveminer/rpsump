#[cfg(test)]
mod tests {
    use rpsump::repository::models::irrigation_event::IrrigationEventStatus;
    use rpsump::repository::Repo;
    use rpsump::test_fixtures::gpio::build_mock_gpio;

    use std::error::Error;
    use tokio::time::Duration;

    use crate::common::fixtures::irrigation_schedule::{
        insert_eligible_schedule_first_run, insert_eligible_schedule_subsequent_run,
        insert_finished_schedule, insert_inactive_schedule, insert_not_today_schedule,
        insert_pending_schedule,
    };
    use crate::common::test_app::spawn_app;

    #[tokio::test]
    async fn test_irrigation_schedule() -> Result<(), Box<dyn Error>> {
        let app = spawn_app(&build_mock_gpio()).await;

        insert_test_data(app.repo).await;

        let events_before = app.repo.irrigation_events().await?;
        let queued_events = events_before
            .iter()
            .filter(|e| e.status == IrrigationEventStatus::Queued.to_string());

        let in_prog_events = events_before
            .iter()
            .filter(|e| e.status == IrrigationEventStatus::InProgress.to_string());

        let completed_events = events_before
            .iter()
            .filter(|e| e.status == IrrigationEventStatus::Completed.to_string());

        assert!(queued_events.count() == 0);
        assert!(in_prog_events.count() == 0);
        // Finished, Not Today, and Subsequent Run scheduled have events
        assert!(completed_events.count() == 5);

        // Wait long enough for two 1-second jobs to complete
        tokio::time::sleep(Duration::from_secs(15)).await;

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

        assert!(queued_events == 0);
        assert!(in_prog_events == 0);
        assert!(completed_events == 8);

        Ok(())
    }

    async fn insert_test_data(repo: Repo) {
        // Eligible but inactive
        insert_inactive_schedule(repo).await;

        // Scheduled time not reached yet
        insert_pending_schedule(repo).await;

        // Event already ran today
        insert_finished_schedule(repo).await;

        // Not scheduled today
        insert_not_today_schedule(repo).await;

        // Eligible
        insert_eligible_schedule_first_run(repo).await;
        insert_eligible_schedule_subsequent_run(repo).await;
    }
}
