use chrono::NaiveDateTime;
use diesel::{prelude::*, sql_types::Text, AsExpression};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::schema::user_event;
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
#[diesel(sql_type = Text)]
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
