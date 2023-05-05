use actix_web::web;
use anyhow::{anyhow, Error};
use diesel::backend::Backend;
use diesel::dsl::*;
use diesel::prelude::*;

use crate::database::DbPool;
use crate::schema::sump_event;
use crate::schema::sump_event::dsl::*;

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = sump_event)]
pub struct SumpEvent {
    pub id: i32,
    pub kind: String,
    pub info: String,
    pub created_at: String,
    pub updated_at: String,
}

impl SumpEvent {
    pub async fn create(
        event_kind: String,
        event_info: String,
        db: DbPool,
    ) -> Result<SumpEvent, Error> {
        let new_sump_event: SumpEvent = web::block(move || {
            let mut conn = db.get().expect("Could not get a db connection.");

            diesel::insert_into(sump_event::table)
                .values((kind.eq(event_kind), info.eq(event_info)))
                .get_result(&mut conn)
        })
        .await?
        .map_err(|_| anyhow!("Internal server error when creating user."))?;

        Ok(new_sump_event)
    }

    pub fn all<DB>() -> Select<sump_event::table, AsSelect<SumpEvent, DB>>
    where
        DB: Backend,
    {
        sump_event::table.select(SumpEvent::as_select())
    }
}
