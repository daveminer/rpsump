use actix_web::web::Data;
use anyhow::{anyhow, Error};
use chrono::{NaiveDateTime, Utc};
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::{prelude::*, sqlite::Sqlite};
use serde::{Deserialize, Serialize};

use crate::models::user_event::EventType;
use crate::schema::{user, user_event};
use crate::util::spawn_blocking_with_tracing;

#[derive(Clone, Debug, PartialEq, Queryable, Selectable, Serialize, Deserialize)]
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

    // #[tracing::instrument(name = "Create user", skip(new_email, new_password, db))]
    // pub async fn create<D>(
    //     new_email: String,
    //     new_password: String,
    //     req_ip_address: String,
    //     db: Data<D>,
    // ) -> Result<User, Error>
    // where
    //     D: DbPool + Send + Sync + ?Sized + 'static,
    // {
    //     let new_user: User = spawn_blocking_with_tracing(move || {
    //         let mut conn = db.get_conn()?;

    //         let user = conn.transaction::<_, Error, _>(|conn| {
    //             let _row_inserted = diesel::insert_into(user::table)
    //                 .values((
    //                     user::email.eq(new_email.clone()),
    //                     user::password_hash.eq(new_password),
    //                 ))
    //                 .execute(conn)
    //                 .map_err(|e| match e {
    //                     DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
    //                         anyhow!("Email already exists.")
    //                     }
    //                     e => anyhow!("Internal server error when creating user: {}", e),
    //                 })?;

    //             let user = user::table
    //                 .filter(user::email.eq(new_email))
    //                 .first::<User>(conn)
    //                 .map_err(|e| anyhow!("Error when fetching user: {}", e))?;

    //             let _user_event_row_inserted = diesel::insert_into(user_event::table)
    //                 .values((
    //                     user_event::user_id.eq(user.id),
    //                     user_event::event_type.eq(EventType::Signup.to_string()),
    //                     user_event::ip_address.eq(req_ip_address.clone()),
    //                 ))
    //                 .execute(conn)?;

    //             Ok(user)
    //         });

    //         user
    //     })
    //     .await??;

    //     Ok(new_user)
    // }

    // #[tracing::instrument(skip(self, password, db))]
    // pub async fn set_password<D>(self, password: &Password, db: Data<D>) -> Result<(), Error>
    // where
    //     D: DbPool + Send + Sync + ?Sized + 'static,
    // {
    //     let hash = password.hash()?;
    //     let _row_updated = spawn_blocking_with_tracing(move || {
    //         let mut conn = db.get_conn()?;

    //         diesel::update(user::table)
    //             .filter(user::email.eq(self.email))
    //             .set(user::password_hash.eq(hash))
    //             .execute(&mut conn)
    //             .map_err(|e| anyhow!(e))
    //     })
    //     .await?
    //     .map_err(|e| anyhow!("Error when updating user password: {}", e))?;

    //     Ok(())
    // }

    // #[tracing::instrument(skip(self, token, db))]
    // pub async fn save_reset_token<D>(self, token: Token, db: Data<D>) -> Result<(), Error>
    // where
    //     D: DbPool + Send + Sync + ?Sized + 'static,
    // {
    //     let _row_updated = spawn_blocking_with_tracing(move || {
    //         let mut conn = db.get_conn()?;

    //         diesel::update(user::table)
    //             .filter(user::email.eq(self.email))
    //             .set((
    //                 user::password_reset_token.eq(token.value),
    //                 user::password_reset_token_expires_at.eq(token.expires_at.to_string()),
    //             ))
    //             .execute(&mut conn)
    //             .map_err(|e| anyhow!(e))
    //     })
    //     .await?
    //     .map_err(|e| anyhow!("Error when saving new reset token: {}", e))?;

    //     Ok(())
    // }

    // #[tracing::instrument(skip(user_email, token, db))]
    // pub async fn save_email_verification_token<D>(
    //     user_email: String,
    //     token: Token,
    //     db: Data<D>,
    // ) -> Result<(), Error>
    // where
    //     D: DbPool + Send + Sync + ?Sized + 'static,
    // {
    //     let _row_updated = spawn_blocking_with_tracing(move || {
    //         let mut conn = db.get_conn()?;

    //         diesel::update(user::table)
    //             .filter(user::email.eq(user_email))
    //             .set((
    //                 user::email_verification_token.eq(token.value),
    //                 user::email_verification_token_expires_at.eq(Some(token.expires_at)),
    //             ))
    //             .execute(&mut conn)
    //             .map_err(|e| anyhow!(e))
    //     })
    //     .await?
    //     .map_err(|e| anyhow!("Error when saving email verification token: {}", e))?;

    //     Ok(())
    // }

    // #[tracing::instrument]
    // fn check_email_verification_expiry(expires_at: Option<NaiveDateTime>) -> Result<(), Error> {
    //     match expires_at {
    //         Some(expires_at) => {
    //             if expires_at <= Utc::now().naive_utc() {
    //                 return Err(anyhow!("Token expired."));
    //             }

    //             return Ok(());
    //         }
    //         None => return Err(anyhow!("Invalid token.")),
    //     }
    // }
}
