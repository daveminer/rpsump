use diesel::prelude::*;

use crate::schema::sump_events;

#[derive(Debug, Queryable)]
pub struct SumpEvent {
    pub id: i32,
    pub kind: String,
    pub info: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Insertable)]
#[diesel(table_name = sump_events)]
pub struct NewSumpEvent<'a> {
    pub kind: &'a str,
    pub info: &'a str,
}
