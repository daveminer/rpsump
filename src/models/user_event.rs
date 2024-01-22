use actix_web::web::Data;
use anyhow::{anyhow, Error};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::AsExpression;
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::models::user::User;
use crate::schema::user_event;
use crate::schema::user_event::dsl::*;
use crate::util::spawn_blocking_with_tracing;

#[derive(Clone, PartialEq, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = user_event)]
pub struct UserEvent {
    pub id: i32,
    pub user_id: i32,
    pub event_type: String,
    pub ip_address: String,
    pub created_at: NaiveDateTime,
}

#[derive(Clone, Debug, DbEnum, AsExpression, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[diesel(sql_type = diesel::sql_types::Text)]
pub enum EventType {
    FailedLogin,
    Login,
    Logout,
    PasswordReset,
    Signup,
}

impl Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::FailedLogin => write!(f, "failed_login"),
            EventType::Login => write!(f, "login"),
            EventType::Logout => write!(f, "logout"),
            EventType::PasswordReset => write!(f, "password_reset"),
            EventType::Signup => write!(f, "signup"),
        }
    }
}

// impl UserEvent {
//     #[tracing::instrument(skip(request_user, db))]
//     pub async fn recent_events<D>(
//         request_user: Option<User>,
//         ip_addr: Option<String>,
//         event: EventType,
//         count: i64,
//         db: Data<D>,
//     ) -> Result<Vec<UserEvent>, Error>
//     where
//         D: DbPool + Send + Sync + ?Sized + 'static,
//     {
//         if request_user.is_none() && ip_addr.is_none() {
//             return Err(anyhow!("Must provide either a user or ip address."));
//         }

//         let mut query = user_event
//             .filter(event_type.eq(event.to_string()))
//             .into_boxed();

//         if request_user.is_some() {
//             query = query.filter(user_id.eq(request_user.unwrap().id));
//         }

//         if ip_addr.is_some() {
//             query = query.filter(ip_address.eq(ip_addr.unwrap()));
//         }

//         spawn_blocking_with_tracing(move || {
//             let mut conn = db.get_conn().expect("Could not get a db connection.");

//             query.limit(count).load(&mut conn)
//         })
//         .await?
//         .map_err(|e| {
//             anyhow!(
//                 "Internal server error when getting recent user events: {}",
//                 e
//             )
//         })
//     }

//     #[tracing::instrument(skip(request_user, db))]
//     pub async fn create<D>(
//         request_user: User,
//         request_ip_address: String,
//         user_event_type: EventType,
//         db: Data<D>,
//     ) -> Result<usize, Error>
//     where
//         D: DbPool + Send + Sync + ?Sized + 'static,
//     {
//         let new_user_event = spawn_blocking_with_tracing(move || {
//             let mut conn = db.get_conn().expect("Could not get a db connection.");

//             diesel::insert_into(user_event::table)
//                 .values((
//                     user_id.eq(request_user.id),
//                     event_type.eq(user_event_type.to_string()),
//                     ip_address.eq(request_ip_address),
//                 ))
//                 .execute(&mut conn)
//         })
//         .await?
//         .map_err(|e| anyhow!("Internal server error when creating user event: {}", e))?;

//         Ok(new_user_event)
//     }
// }
