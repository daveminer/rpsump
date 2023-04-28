use actix_web::{error, web};
use anyhow::{anyhow, Error};
use chrono::{Duration, Utc};
use diesel::prelude::*;
use diesel::sqlite::Sqlite;
use serde::{Deserialize, Serialize};

use crate::database::DbPool;
use crate::schema::user;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Queryable, Selectable)]
#[diesel(table_name = user)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub email_verification_token: Option<String>,
    pub email_verified_at: Option<String>,
    pub password_hash: String,
    pub password_reset_token_hash: Option<String>,
    pub password_reset_token_expires_at: Option<String>,
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
    pub async fn create(self, db: DbPool) -> Result<(), Error> {
        let _row_inserted = web::block(move || {
            let mut conn = db.get().expect("Could not get a db connection.");

            diesel::insert_into(user::table)
                .values(self)
                .execute(&mut conn)
        })
        .await?
        .map_err(|_| anyhow!("Internal server error when creating user."))?;

        Ok(())
    }
}

impl User {
    pub fn by_email(email: String) -> BoxedQuery<'static> {
        user::table.filter(user::email.eq(email)).into_boxed()
    }

    pub async fn save_reset_token(
        self,
        token_hash: String,
        db: actix_web::web::Data<DbPool>,
    ) -> Result<(), Error> {
        let _row_updated = web::block(move || {
            let mut conn = db.get().expect("Could not get a db connection.");

            let expires_at = Utc::now() + Duration::hours(2);

            diesel::update(user::table)
                .filter(user::email.eq(self.email))
                .set((
                    user::password_reset_token_hash.eq(token_hash),
                    user::password_reset_token_expires_at.eq(expires_at.to_string()),
                ))
                .execute(&mut conn)
        })
        .await?
        .map_err(|_| anyhow!("Internal server error when creating user."))?;

        Ok(())
    }

    pub async fn save_email_verification_token(
        self,
        token: String,
        db: actix_web::web::Data<DbPool>,
    ) -> Result<(), Error> {
        let _row_updated = web::block(move || {
            let mut conn = db.get().expect("Could not get a db connection.");

            diesel::update(user::table)
                .filter(user::email.eq(self.email))
                .set(user::email_verification_token.eq(token))
                .execute(&mut conn)
        })
        .await?
        .map_err(|_| anyhow!("Internal server error when saving email verification token."))?;

        Ok(())
    }

    pub async fn verify_email(
        self,
        token: String,
        db: actix_web::web::Data<DbPool>,
    ) -> Result<(), Error> {
        let _row_updated = web::block(move || {
            let mut conn = db.get().expect("Could not get a db connection.");

            diesel::update(user::table)
                .filter(user::email_verification_token.eq(token))
                .set(user::email_verified_at.eq(Utc::now().to_string()))
                .execute(&mut conn)
        })
        .await?
        .map_err(|_| anyhow!("Internal server error when verifying email token."))?;

        Ok(())
    }
}
