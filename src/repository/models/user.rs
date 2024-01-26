use chrono::NaiveDateTime;
use diesel::{prelude::*, sqlite::Sqlite};
use serde::{Deserialize, Serialize};

use crate::schema::user;

#[derive(Clone, Debug, Identifiable, PartialEq, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = user)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub email_verification_token: Option<String>,
    pub email_verification_token_expires_at: Option<NaiveDateTime>,
    pub email_verified_at: Option<NaiveDateTime>,
    pub password_hash: String,
    pub password_reset_token: Option<String>,
    pub password_reset_token_expires_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Default)]
pub struct UserFilter {
    pub id: Option<i32>,
    pub email: Option<String>,
    pub email_verification_token: Option<String>,
    pub password_hash: Option<String>,
    pub password_reset_token: Option<String>,
}

#[derive(AsChangeset)]
#[diesel(table_name = user)]
pub struct UserUpdateFilter {
    pub id: i32,
    pub email: Option<String>,
    pub email_verification_token: Option<Option<String>>,
    pub password_hash: Option<String>,
    pub password_reset_token: Option<Option<String>>,
}

type BoxedQuery<'a> = user::BoxedQuery<'a, Sqlite, user::SqlType>;
