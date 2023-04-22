use diesel::prelude::*;
use diesel::sqlite::Sqlite;
use serde::{Deserialize, Serialize};

use crate::database::DbPool;
use crate::schema::*;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = user)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub password_hash: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = user)]
pub struct NewUser {
    pub email: String,
    pub password_hash: String,
}

type BoxedQuery<'a> = user::BoxedQuery<'a, Sqlite, user::SqlType>;

impl NewUser {
    pub fn create(self, db: DbPool) -> usize {
        let mut conn = db.get().expect("Could not get a db connection.");

        diesel::insert_into(user::table)
            .values(self)
            .execute(&mut conn)
            .expect("Error saving sump event")
    }
}

impl User {
    pub fn by_email(email: String) -> BoxedQuery<'static> {
        user::table.filter(user::email.eq(email)).into_boxed()
    }
}
