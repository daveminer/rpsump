use actix_web::web;
use anyhow::{anyhow, Error};
use chrono::NaiveDateTime;
use diesel::backend::Backend;
use diesel::dsl::*;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::database::DbPool;
use crate::schema::sump_event;
use crate::schema::sump_event::*;

#[derive(Clone, PartialEq, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = sump_event)]
pub struct SumpEvent {
    pub id: i32,
    pub kind: String,
    pub info: String,
    pub created_at: NaiveDateTime,
}

impl SumpEvent {
    pub async fn create(event_kind: String, event_info: String, db: DbPool) -> Result<(), Error> {
        web::block(move || {
            let mut conn = db.get().expect("Could not get a db connection.");

            diesel::insert_into(sump_event::table)
                .values((kind.eq(event_kind), info.eq(event_info)))
                .execute(&mut conn)
        })
        .await?
        .map_err(|e| anyhow!("Internal server error when creating sump event: {e}"))?;

        Ok(())
    }

    pub fn all<DB>() -> Select<sump_event::table, AsSelect<SumpEvent, DB>>
    where
        DB: Backend,
    {
        sump_event::table.select(SumpEvent::as_select())
    }
}
