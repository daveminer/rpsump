use rpsump::repository::Repo;

/// Inserts a SumpEvent directly into the database, bypassing any application logic.
async fn insert_sump_event(repo: Repo, event_kind: String, event_info: String) {
    repo.create_sump_event(event_info, event_kind)
        .await
        .unwrap();
}

pub async fn insert_sump_events(repo: Repo) {
    insert_sump_event(repo, "sump pump".to_string(), "pump on".to_string()).await;
    insert_sump_event(repo, "sump pump".to_string(), "pump off".to_string()).await;
    insert_sump_event(repo, "sump pump".to_string(), "pump on".to_string()).await;
    insert_sump_event(repo, "sump pump".to_string(), "pump off".to_string()).await;
}
