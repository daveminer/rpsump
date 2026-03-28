use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::refresh_token;

#[derive(Clone, Debug, Identifiable, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = refresh_token)]
pub struct RefreshToken {
    pub id: i32,
    pub user_id: i32,
    pub token: String,
    pub expires_at: NaiveDateTime,
    pub revoked_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}
