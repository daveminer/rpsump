use actix_web::{web, web::Data};
use anyhow::{anyhow, Error};
use diesel::prelude::*;
use diesel::AsExpression;
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::database::DbPool;
use crate::models::user::User;
use crate::schema::user_event;
use crate::schema::user_event::dsl::*;

#[derive(Clone, Debug, PartialEq, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = user_event)]
pub struct UserEvent {
    pub id: i32,
    pub user_id: i32,
    pub event_type: String,
    pub ip_address: String,
    pub created_at: String,
}

#[derive(Clone, Debug, DbEnum, AsExpression, Serialize, Deserialize)]
#[sql_type = "diesel::sql_types::Text"]
pub enum EventType {
    Login,
    Logout,
    PasswordReset,
    Signup,
    UpdateProfile,
}

impl Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::Login => write!(f, "login"),
            EventType::Logout => write!(f, "logout"),
            EventType::PasswordReset => write!(f, "password_reset"),
            EventType::Signup => write!(f, "signup"),
            EventType::UpdateProfile => write!(f, "update_profile"),
        }
    }
}

impl UserEvent {
    pub async fn create(
        user: User,
        request_ip_address: String,
        user_event_type: EventType,
        db: Data<DbPool>,
    ) -> Result<UserEvent, Error> {
        let new_user_event: UserEvent = web::block(move || {
            let mut conn = db.get().expect("Could not get a db connection.");

            diesel::insert_into(user_event::table)
                .values((
                    user_id.eq(user.id),
                    event_type.eq(user_event_type.to_string()),
                    ip_address.eq(request_ip_address),
                ))
                .get_result(&mut conn)
        })
        .await?
        .map_err(|_| anyhow!("Internal server error when creating user event."))?;

        Ok(new_user_event)
    }
}
