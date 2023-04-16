use anyhow::Error;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::sqlite::SqliteConnection;
use std::sync::{Arc, Mutex};

use crate::models::*;
use crate::schema::{sump_events, sump_events::dsl::*};

#[derive(Clone)]
pub struct Database {
    pub pool: Arc<Mutex<Pool<ConnectionManager<SqliteConnection>>>>,
}

impl Database {
    pub fn new(path: String) -> Result<Database, Error> {
        let manager = ConnectionManager::<SqliteConnection>::new(&path);
        let pool = Pool::builder().build(manager)?;

        Ok(Database {
            pool: Arc::new(Mutex::new(pool)),
        })
    }

    pub fn create_sump_event(self, event_kind: &str, event_info: &str) -> usize {
        let new_sump_event = NewSumpEvent {
            kind: event_kind,
            info: event_info,
        };

        diesel::insert_into(sump_events::table)
            .values(&new_sump_event)
            .execute(&mut self.conn())
            .expect("Could not save new sump_event.")
    }

    pub fn get_sump_events(self) -> Vec<SumpEvent> {
        sump_events
            .limit(100)
            .load::<SumpEvent>(&mut self.conn())
            .expect("Could not query sump_events")
    }

    fn conn(self) -> PooledConnection<ConnectionManager<SqliteConnection>> {
        self.pool.lock().unwrap().get().unwrap()
    }
}

impl std::fmt::Debug for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Database")
            .field("diesel", &"sqlite")
            .finish()
    }
}
