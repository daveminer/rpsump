use diesel::backend::Backend;
use diesel::dsl::*;
use diesel::prelude::*;

use crate::database::DbPool;
use crate::schema::*;

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = sump_event)]
pub struct SumpEvent {
    pub id: i32,
    pub kind: String,
    pub info: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = sump_event)]
pub struct NewSumpEvent<'a> {
    pub kind: &'a str,
    pub info: &'a str,
}

impl<'a> NewSumpEvent<'a> {
    pub fn create(self, db: DbPool) -> usize {
        let mut conn = db.get().expect("Could not get a db connection.");

        diesel::insert_into(sump_event::table)
            .values(self)
            .execute(&mut conn)
            .expect("Error saving sump event")
    }
}

impl SumpEvent {
    pub fn all<DB>() -> Select<sump_event::table, AsSelect<SumpEvent, DB>>
    where
        DB: Backend,
    {
        sump_event::table.select(SumpEvent::as_select())
    }
}
