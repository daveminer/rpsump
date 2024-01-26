use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::sump_event;

#[derive(Clone, Debug, PartialEq, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = sump_event)]
pub struct SumpEvent {
    pub id: i32,
    pub kind: String,
    pub info: String,
    pub created_at: NaiveDateTime,
}
