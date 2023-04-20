use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::*;

#[derive(Debug, Clone, Deserialize, Serialize, Queryable)]
#[diesel(table_name = user)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub password_hash: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Insertable, Deserialize, Serialize, AsChangeset)]
#[diesel(table_name = user)]
pub struct NewUser {
    pub email: String,
    pub password_hash: String,
}
