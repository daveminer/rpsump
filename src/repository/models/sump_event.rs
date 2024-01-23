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

// impl SumpEvent {
//     pub async fn create(
//         event_kind: String,
//         event_info: String,
//         db: RealDbPool,
//     ) -> Result<(), Error> {
//         let db = db.clone();
//         spawn_blocking_with_tracing(move || {
//             let mut conn = db.get().expect("Could not get a db connection.");

//             diesel::insert_into(sump_event::table)
//                 .values((kind.eq(event_kind), info.eq(event_info)))
//                 .execute(&mut conn)
//         })
//         .await?
//         .map_err(|e| anyhow!("Internal server error when creating sump event: {e}"))?;

//         Ok(())
//     }

//     pub fn all<DB>() -> Select<sump_event::table, AsSelect<SumpEvent, DB>>
//     where
//         DB: Backend,
//     {
//         sump_event.select(SumpEvent::as_select())

//         //sump_event.select(SumpEvent::as_select()).limit(100)
//         //     SumpEvent::table()
//         // .take(100)
//         // .load(conn)
//     }
// }
