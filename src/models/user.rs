use actix_web::{web, web::Data};
use anyhow::{anyhow, Error};
use chrono::Utc;
use diesel::prelude::*;
use diesel::sqlite::Sqlite;
use serde::{Deserialize, Serialize};

use crate::auth::{password::Password, token::Token};
use crate::database::DbPool;
use crate::models::user_event::{EventType, UserEvent};
use crate::schema::{user, user_event};

#[derive(Clone, Debug, PartialEq, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = user)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub email_verification_token: Option<String>,
    pub email_verification_token_expires_at: Option<String>,
    pub email_verified_at: Option<String>,
    pub password_hash: String,
    pub password_reset_token: Option<String>,
    pub password_reset_token_expires_at: Option<String>,
    pub activated: bool,
    pub created_at: String,
    pub updated_at: String,
}

type BoxedQuery<'a> = user::BoxedQuery<'a, Sqlite, user::SqlType>;

impl User {
    // Composable queries
    pub fn by_email(user_email: String) -> BoxedQuery<'static> {
        user::table.filter(user::email.eq(user_email)).into_boxed()
    }

    pub fn by_email_verification_token(user_token: String) -> BoxedQuery<'static> {
        user::table
            .filter(user::email_verification_token.eq(user_token))
            .into_boxed()
    }

    pub fn by_id(user_id: i32) -> BoxedQuery<'static> {
        user::table.filter(user::id.eq(user_id)).into_boxed()
    }

    pub fn by_password_reset_token(user_token: String) -> BoxedQuery<'static> {
        user::table
            .filter(user::password_reset_token.eq(user_token))
            .into_boxed()
    }

    pub async fn create(
        new_email: String,
        new_password: String,
        req_ip_address: String,
        db: Data<DbPool>,
    ) -> Result<User, Error> {
        let new_user: User = web::block(move || {
            let mut conn = db.get().expect("Could not get a db connection.");

            let user = conn.transaction::<_, Error, _>(|conn| {
                let user: User = diesel::insert_into(user::table)
                    .values((
                        user::email.eq(new_email.clone()),
                        user::password_hash.eq(new_password),
                    ))
                    .get_result(conn)?;

                let _user_event: UserEvent = diesel::insert_into(user_event::table)
                    .values((
                        user_event::user_id.eq(user.id),
                        user_event::event_type.eq(EventType::Signup.to_string()),
                        user_event::ip_address.eq(req_ip_address.clone()),
                    ))
                    .get_result(conn)?;

                Ok(user)
            });

            user
        })
        .await?
        .map_err(|e| {
            println!("Error creating user: {:?}", e);
            return anyhow!("Internal server error when creating user.");
        })?;

        Ok(new_user)
    }

    pub async fn set_password(self, password: &Password, db: Data<DbPool>) -> Result<(), Error> {
        let hash = password.hash()?;
        let _row_updated = web::block(move || {
            let mut conn = db.get().expect("Could not get a db connection.");

            diesel::update(user::table)
                .filter(user::email.eq(self.email))
                .set(user::password_hash.eq(hash))
                .execute(&mut conn)
        })
        .await?
        .map_err(|_| anyhow!("Internal server error when creating user."))?;

        Ok(())
    }

    pub async fn save_reset_token(self, token: Token, db: Data<DbPool>) -> Result<(), Error> {
        let _row_updated = web::block(move || {
            let mut conn = db.get().expect("Could not get a db connection.");

            diesel::update(user::table)
                .filter(user::email.eq(self.email))
                .set((
                    user::password_reset_token.eq(token.value),
                    user::password_reset_token_expires_at.eq(token.expires_at.to_string()),
                ))
                .execute(&mut conn)
        })
        .await?
        .map_err(|_| anyhow!("Internal server error when creating user."))?;

        Ok(())
    }

    pub async fn save_email_verification_token(
        user_email: String,
        token: Token,
        db: web::Data<DbPool>,
    ) -> Result<(), Error> {
        let _row_updated = web::block(move || {
            let mut conn = db.get().expect("Could not get a db connection.");

            diesel::update(user::table)
                .filter(user::email.eq(user_email))
                .set((
                    user::email_verification_token.eq(token.value),
                    user::email_verification_token_expires_at.eq(token.expires_at.to_string()),
                ))
                .execute(&mut conn)
        })
        .await?
        .map_err(|_| anyhow!("Internal server error when saving email verification token."))?;

        Ok(())
    }

    pub async fn verify_email(token: String, db: web::Data<DbPool>) -> Result<(), Error> {
        let _result = web::block(move || {
            let mut conn = db.get().expect("Could not get a db connection.");

            let user_from_token = Self::by_email_verification_token(token.clone())
                .first::<User>(&mut conn)
                .map_err(|_| anyhow!("Invalid token."))?;

            if user_from_token.email_verified_at.is_some() {
                return Err(anyhow!("Email already verified."));
            }

            if let Err(e) = Self::check_email_verification_expiry(
                user_from_token.email_verification_token_expires_at,
            ) {
                return Err(e);
            }

            diesel::update(user::table)
                .filter(user::email_verification_token.eq(token))
                .set((
                    user::email_verification_token.eq(None::<String>),
                    user::email_verification_token_expires_at.eq(None::<String>),
                    user::email_verified_at.eq(Utc::now().to_string()),
                ))
                .execute(&mut conn)?;

            Ok(())
        })
        .await?;

        Ok(())
    }

    fn check_email_verification_expiry(expires_at: Option<String>) -> Result<(), Error> {
        let expires_at = expires_at
            .ok_or_else(|| anyhow!("Invalid token."))?
            .parse::<chrono::DateTime<chrono::Utc>>()
            .map_err(|_| anyhow!("Invalid token."))?;

        if expires_at <= Utc::now() {
            return Err(anyhow!("Token expired."));
        }

        Ok(())
    }
}
