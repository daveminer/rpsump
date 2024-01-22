use diesel::prelude::*;
use rpsump::repository::DbPool;
use rpsump::schema::sump_event;
use rpsump::schema::sump_event::dsl::*;

/// Inserts a SumpEvent directly into the database, bypassing any application logic.
async fn insert_sump_event(db: DbPool, event_kind: String, event_info: String) {
    let mut conn = db.get().unwrap();
    diesel::insert_into(sump_event::table)
        .values((kind.eq(event_kind), info.eq(event_info)))
        .execute(&mut conn)
        .unwrap();
}

pub async fn insert_sump_events(db: DbPool) {
    insert_sump_event(db.clone(), "sump pump".to_string(), "pump on".to_string()).await;
    insert_sump_event(db.clone(), "sump pump".to_string(), "pump off".to_string()).await;
    insert_sump_event(db.clone(), "sump pump".to_string(), "pump on".to_string()).await;
    insert_sump_event(db.clone(), "sump pump".to_string(), "pump off".to_string()).await;
}
