use std::collections::HashMap;

use anyhow::{anyhow, Error};
use async_trait::async_trait;
use chrono::Utc;
use diesel::query_dsl::methods::{FilterDsl, LimitDsl};
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::sqlite::SqliteConnection;
use diesel::{Connection, ExpressionMethods, RunQueryDsl};

use crate::auth::password::Password;
use crate::auth::token::Token;
use crate::repository::models::{
    irrigation_event::{IrrigationEvent, IrrigationEventStatus, StatusQueryResult},
    irrigation_schedule::IrrigationSchedule,
    sump_event::SumpEvent,
    user::User,
    user_event::EventType,
};
use crate::repository::Repository;
use crate::schema::{irrigation_event, irrigation_schedule, user, user_event};
use crate::schema::{
    irrigation_event::dsl as irrigation_event_dsl,
    irrigation_schedule::dsl as irrigation_schedule_dsl, sump_event::dsl as sump_event_dsl,
};
use crate::util::spawn_blocking_with_tracing;

pub type DbConn = PooledConnection<ConnectionManager<SqliteConnection>>;
type DbPool = Pool<ConnectionManager<SqliteConnection>>;

pub struct Implementation {
    pool: DbPool,
}

impl Implementation {
    pub async fn new(database_uri: String) -> Result<Self, Error> {
        let manager = ConnectionManager::<SqliteConnection>::new(database_uri);
        let pool = Pool::new(manager)?;

        Ok(Implementation { pool })
    }
}

#[async_trait]
impl Repository for Implementation {
    async fn create_irrigation_event(
        &self,
        schedule: IrrigationSchedule,
        hose: i32,
    ) -> Result<(), Error> {
        spawn_blocking_with_tracing(move || {
            let mut conn = self
                .pool
                .get()
                .map_err(|e| anyhow!("Database error: {:?}", e))?;

            diesel::insert_into(irrigation_event::table)
                .values((
                    irrigation_event_dsl::schedule_id.eq(schedule.id),
                    irrigation_event_dsl::hose_id.eq(hose),
                    irrigation_event_dsl::status.eq(IrrigationEventStatus::Queued.to_string()),
                ))
                .execute(&mut conn)
                .map_err(|e| anyhow!("Error creating irrigation event: {}", e))
        })
        .await?
        .map_err(|e| anyhow!("Internal server error when creating irrigation event: {e}"))?;

        Ok(())
    }

    async fn create_irrigation_schedule(
        &self,
        // TODO: make type
        params: HashMap<String, String>,
    ) -> Result<(), Error> {
        let _irrigation_sched = spawn_blocking_with_tracing(move || {
            let mut conn = self
                .pool
                .get()
                .map_err(|e| anyhow!("Database error: {:?}", e))?;

            let active: bool = params.get("active").unwrap().parse().unwrap();
            let duration: i32 = params.get("duration").unwrap().parse().unwrap();

            diesel::insert_into(irrigation_schedule::table)
                .values((
                    irrigation_schedule_dsl::active.eq(active),
                    irrigation_schedule_dsl::name.eq(params.get("name").unwrap()),
                    irrigation_schedule_dsl::duration.eq(duration),
                    irrigation_schedule_dsl::start_time.eq(params.get("start_time").unwrap()),
                    irrigation_schedule_dsl::days_of_week.eq(params.get("days_of_week").unwrap()),
                    irrigation_schedule_dsl::hoses.eq(params.get("hoses").unwrap()),
                    //TODO: check created at
                ))
                .execute(&mut conn)
                .map_err(|e| anyhow!("Error creating irrigation schedule: {}", e))
        })
        .await??;

        Ok(())
    }

    async fn create_password_reset(&self, current_user: User) -> Result<(), Error> {
        let _row_updated = spawn_blocking_with_tracing(move || {
            let mut conn = self
                .pool
                .get()
                .map_err(|e| anyhow!("Database error: {:?}", e))?;

            let token = Token::new_password_reset(current_user.id);

            diesel::update(user::table)
                .filter(user::email.eq(current_user.email))
                .set((
                    user::password_reset_token.eq(token.value),
                    user::password_reset_token_expires_at.eq(token.expires_at),
                ))
                .execute(&mut conn)
                .map_err(|e| anyhow!(e))
        })
        .await?
        .map_err(|e| anyhow!("Error when updating user password: {}", e))?;

        Ok(())
    }

    // fn create_sump_event(
    //     &self,
    //     sump_id: i32,
    //     event_type: String,
    //     ip_address: String,
    // ) -> Result<SumpEvent, Error>;
    async fn create_user(
        &self,
        new_email: String,
        new_password_hash: String,
        req_ip_address: String,
    ) -> Result<User, Error> {
        let new_user: User = spawn_blocking_with_tracing(move || {
            let mut conn = self
                .pool
                .get()
                .map_err(|e| anyhow!("Database error: {:?}", e))?;

            let user = conn.transaction::<_, Error, _>(|conn| {
                let _row_inserted = diesel::insert_into(user::table)
                    .values((
                        user::email.eq(new_email.clone()),
                        user::password_hash.eq(new_password_hash),
                    ))
                    .execute(conn)
                    .map_err(|e| match e {
                        DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                            anyhow!("Email already exists.")
                        }
                        e => anyhow!("Internal server error when creating user: {}", e),
                    })?;

                let user = user::table
                    .filter(user::email.eq(new_email))
                    .first::<User>(conn)
                    .map_err(|e| anyhow!("Error when fetching user: {}", e))?;

                let _user_event_row_inserted = diesel::insert_into(user_event::table)
                    .values((
                        user_event::user_id.eq(user.id),
                        user_event::event_type.eq(EventType::Signup.to_string()),
                        user_event::ip_address.eq(req_ip_address.clone()),
                    ))
                    .execute(conn)?;

                Ok(user)
            });

            user
        })
        .await??;

        Ok(new_user)
    }

    async fn create_user_event(
        &self,
        user: &User,
        request_event_type: EventType,
        request_ip_address: String,
    ) -> Result<(), Error> {
        let _new_user_event = spawn_blocking_with_tracing(move || {
            let mut conn = self
                .pool
                .get()
                .map_err(|e| anyhow!("Database error: {:?}", e))?;

            diesel::insert_into(user_event::table)
                .values((
                    user_event::user_id.eq(user.id),
                    user_event::event_type.eq(request_event_type.to_string()),
                    user_event::ip_address.eq(request_ip_address),
                ))
                .execute(&mut conn)
                .map_err(|e| anyhow!("Error creating user event: {}", e))
        })
        .await?
        .map_err(|e| anyhow!("Internal server error when creating user event: {}", e))?;

        Ok(())
    }

    async fn delete_irrigation_schedule(&self, sched_id: i32) -> Result<(), Error> {
        Ok(())
    }

    async fn irrigation_events(&self) -> Result<Vec<IrrigationEvent>, Error> {
        let irrigation_events = spawn_blocking_with_tracing(move || {
            let mut conn = self
                .pool
                .get()
                .map_err(|e| anyhow!("Database error: {:?}", e))?;

            irrigation_event_dsl::irrigation_event
                .limit(100)
                .load::<IrrigationEvent>(&mut conn)
                .map_err(|e| anyhow!(e))
        })
        .await?
        .map_err(|e| {
            anyhow!(
                "Internal server error when getting irrigation events: {}",
                e
            )
        })?;

        Ok(irrigation_events)
    }

    async fn irrigation_schedules(&self) -> Result<Vec<IrrigationSchedule>, Error> {
        let irrigation_schedules = spawn_blocking_with_tracing(move || {
            let mut conn = self
                .pool
                .get()
                .map_err(|e| anyhow!("Database error: {:?}", e))?;

            irrigation_schedule_dsl::irrigation_schedule
                .limit(100)
                .load::<IrrigationSchedule>(&mut conn)
                .map_err(|e| anyhow!(e))
        })
        .await??;

        Ok(irrigation_schedules)
    }

    async fn irrigation_schedule_by_id(&self, sched_id: i32) -> Result<IrrigationSchedule, Error> {
        let irrigation_sched = spawn_blocking_with_tracing(move || {
            let mut conn = self
                .pool
                .get()
                .map_err(|e| anyhow!("Database error: {:?}", e))?;

            irrigation_schedule_dsl::irrigation_schedule
                .filter(irrigation_schedule_dsl::id.eq(sched_id))
                .first::<IrrigationSchedule>(&mut conn)
                .map_err(|e| match e {
                    DieselError::NotFound => anyhow!("Irrigation schedule not found."),
                    e => anyhow!(
                        "Internal server error when fetching irrigation schedule: {}",
                        e
                    ),
                })
        })
        .await??;

        Ok(irrigation_sched)
    }

    // TODO: move Token from auth module
    async fn reset_password(&self, password: &Password, token: String) -> Result<(), Error> {
        let _row_updated = spawn_blocking_with_tracing(move || {
            let mut conn = self
                .pool
                .get()
                .map_err(|e| anyhow!("Database error: {:?}", e))?;

            let current_user = match user::table
                .filter(user::password_reset_token.eq(token))
                .first::<User>(&mut conn)
            {
                Ok(current_user) => current_user,
                Err(e) => match e {
                    DieselError::NotFound => return Err(anyhow!("Invalid token.")),
                    _ => return Err(anyhow!(e)),
                },
            };

            let pw_hash = password.hash()?;

            diesel::update(user::table)
                .filter(user::email.eq(current_user.email))
                .set((
                    user::password_hash.eq(pw_hash),
                    user::password_reset_token.eq::<Option<String>>(None),
                    user::password_reset_token_expires_at.eq::<Option<String>>(None),
                ))
                .execute(&mut conn)
                .map_err(|e| anyhow!(e))
        })
        .await?
        .map_err(|e| anyhow!("Error when updating user password: {}", e))?;

        Ok(())
    }

    async fn schedule_statuses(&self) -> Result<Vec<StatusQueryResult>, Error> {
        let statuses = spawn_blocking_with_tracing(move || {
            let mut conn = self
                .pool
                .get()
                .map_err(|e| anyhow!("Database error: {:?}", e))?;

            IrrigationEvent::status_query()
                .load::<StatusQueryResult>(&mut conn)
                .map_err(|e| anyhow!(e))
        })
        .await?
        .map_err(|e| {
            anyhow!(
                "Internal server error when getting schedule statuses: {}",
                e
            )
        })?;

        Ok(statuses)
    }

    async fn sump_events(&self) -> Result<Vec<SumpEvent>, Error> {
        let sump_events = spawn_blocking_with_tracing(move || {
            let mut conn = self
                .pool
                .get()
                .map_err(|e| anyhow!("Database error: {:?}", e))?;

            sump_event_dsl::sump_event
                .limit(100)
                .load::<SumpEvent>(&mut conn)
                .map_err(|e| anyhow!(e))
        })
        .await??;

        Ok(sump_events)
    }
    // fn update_user(&self, user_id: i32, email: String) -> Result<User, Error>;
    // fn update_irrigation_event(&self, event_id: i32) -> Result<IrrigationEvent, Error>;
    // fn update_irrigation_schedule(&self, schedule_id: i32) -> Result<IrrigationSchedule, Error>;
    // fn user_events(&self, user_id: i32, count: i64) -> Result<Vec<UserEvent>, Error>;
    // fn users(&self) -> Result<Vec<User>, Error>;
    async fn user_by_email(&self, email: String) -> Result<Option<User>, Error> {
        spawn_blocking_with_tracing(move || {
            let mut conn = self
                .pool
                .get()
                .map_err(|e| anyhow!("Database error: {:?}", e))?;

            let result = user::table
                .filter(user::email.eq(email))
                .first::<User>(&mut conn);
            //.map_err(|e| anyhow!("Error when looking up user record: {:?}", e));

            match result {
                Ok(user) => Ok(Some(user)),
                Err(e) => match e {
                    DieselError::NotFound => Ok(None),
                    _ => Err(anyhow!(e)),
                },
            }
        })
        .await?
    }

    async fn validate_login(&self, email: String, password: String) -> Result<User, Error> {
        let user = spawn_blocking_with_tracing(move || {
            let mut conn = self
                .pool
                .get()
                .map_err(|e| anyhow!("Database error: {:?}", e))?;

            user::table
                .filter(user::email.eq(email))
                .first::<User>(&mut conn)
                .map_err(|e| match e {
                    DieselError::NotFound => anyhow!("User not found."),
                    e => anyhow!("Internal server error when fetching user: {}", e),
                })
        })
        .await??;

        Ok(user)
    }

    async fn verify_email(&self, token: String) -> Result<(), Error> {
        let _result = spawn_blocking_with_tracing(move || {
            let mut conn = self
                .pool
                .get()
                .map_err(|e| anyhow!("Database error: {:?}", e))?;

            let user: User = user::table
                .filter(user::email_verification_token.eq(token))
                .first::<User>(&mut conn)
                .map_err(|e| match e {
                    DieselError::NotFound => anyhow!("Invalid token."),
                    e => anyhow!("Internal server error when verifying email: {}", e),
                })?;

            // if user_from_token.email_verified_at.is_some() {
            //     return Err(anyhow!("Email already verified."));
            // }

            // Reverse the comparison so it's not on a String, also remove unwrap
            if user.email_verification_token_expires_at.unwrap() < Utc::now().naive_utc() {
                return Err(anyhow!("Token expired."));
            }

            let _row_update_count = diesel::update(user::table)
                .filter(user::email_verification_token.eq(token))
                .set((
                    user::email_verification_token.eq(None::<String>),
                    user::email_verification_token_expires_at.eq(None::<String>),
                    user::email_verified_at.eq(Utc::now().to_string()),
                ))
                .execute(&mut conn)?;

            Ok(())
        })
        .await??;

        Ok(())
    }
}
