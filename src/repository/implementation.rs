use anyhow::{anyhow, Error};
use async_trait::async_trait;
use chrono::{NaiveDateTime, NaiveTime, Utc};

use crate::auth::password::Password;
use crate::auth::token::Token;
use crate::hydro::schedule::ScheduleStatus;
use crate::repository::models::{
    irrigation_event::{IrrigationEvent, IrrigationEventStatus, StatusQueryResult},
    irrigation_schedule::{
        CreateIrrigationScheduleParams, IrrigationSchedule, UpdateIrrigationScheduleParams,
    },
    sump_event::SumpEvent,
    user::User,
    user::UserFilter,
    user_event::{EventType, UserEvent},
};
use crate::repository::Repository;
use crate::schema::{irrigation_event, irrigation_schedule, sump_event, user, user_event};
use crate::schema::{
    irrigation_event::dsl as irrigation_event_dsl,
    irrigation_schedule::dsl as irrigation_schedule_dsl, sump_event::dsl as sump_event_dsl,
};
use crate::util::spawn_blocking_with_tracing;
use diesel::internal::table_macro::BoxedSelectStatement;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::sql_types::{Bool, Nullable};
use diesel::sqlite::SqliteConnection;
use diesel::{BoxableExpression, JoinOnDsl};
use diesel::{Connection, ExpressionMethods, QueryDsl, RunQueryDsl};

use super::models::irrigation_event::NewIrrigationEvent;
use super::models::user::UserUpdateFilter;

type DbPool = Pool<ConnectionManager<SqliteConnection>>;

#[derive(thiserror::Error, Debug)]
pub enum ResetPasswordError {
    #[error("Database error.")]
    DatabaseError(anyhow::Error),
    #[error("Internal server error.")]
    InternalServerError(anyhow::Error),
    #[error("Invalid password")]
    InvalidPassword(anyhow::Error),
    #[error("Invalid token.")]
    InvalidToken,
    #[error("Token expired.")]
    TokenExpired,
}

#[derive(thiserror::Error, Debug)]
pub enum VerifyEmailError {
    #[error("Database error.")]
    DatabaseError(anyhow::Error),
    #[error("Invalid token.")]
    EmailNotFound,
    #[error("Email already verified.")]
    EmailAlreadyVerified,
    #[error("Internal server error.")]
    InternalServerError(anyhow::Error),
    #[error("Token expired.")]
    TokenExpired,
}

#[derive(Clone)]
pub struct Implementation {
    pub pool: DbPool,
}

#[async_trait]
impl Repository for Implementation {
    async fn begin_irrigation(&self, event: IrrigationEvent) -> Result<(), Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        // In a tx - check last event is valid, create the new irrigation event with in progress
        let _row_updated = spawn_blocking_with_tracing(move || {
            conn.transaction::<_, Error, _>(|conn| {
                // Guard for in progress event
                match irrigation_event::table
                    .filter(
                        irrigation_event::status.eq(IrrigationEventStatus::InProgress.to_string()),
                    )
                    .first::<IrrigationEvent>(conn)
                {
                    Ok(_) => return Err(anyhow!("An event is already in progress")),
                    Err(e) => match e {
                        DieselError::NotFound => (),
                        e => return Err(anyhow!("Error when fetching in progress event: {}", e)),
                    },
                };

                diesel::update(irrigation_event::table)
                    .filter(irrigation_event::id.eq(event.id))
                    .set(irrigation_event::status.eq(IrrigationEventStatus::InProgress.to_string()))
                    .execute(conn)
                    .map_err(|e| anyhow!("Error beginning irrigation event: {}", e))?;

                Ok(())
            })
        })
        .await?;

        Ok(())
    }

    async fn create(path: Option<String>) -> Result<Self, Error> {
        let path = if let Some(path) = path {
            path
        } else {
            ":memory:".to_string()
        };

        let manager = ConnectionManager::<SqliteConnection>::new(path);
        let pool = Pool::builder().max_size(1).build(manager)?;

        Ok(Implementation { pool })
    }

    async fn create_email_verification(&self, user_record: &User) -> Result<Token, Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;
        let user_id = user_record.id;
        let email = user_record.email.clone();

        let token = Token::new_email_verification(user_id);
        let token_value = token.value.clone();
        let _: Result<usize, Error> = spawn_blocking_with_tracing(move || {
            let result = diesel::update(user::table)
                .filter(user::email.eq(email))
                .set((
                    user::email_verification_token.eq::<Option<String>>(Some(token_value)),
                    user::email_verification_token_expires_at
                        .eq::<Option<String>>(Some(token.expires_at.to_string())),
                ))
                .execute(&mut conn)
                .map_err(|e| anyhow!("Error when updating user: {}", e))?;

            Ok(result)
        })
        .await?;

        Ok(token)
    }

    async fn create_irrigation_event(
        &self,
        schedule: IrrigationSchedule,
        hose: i32,
    ) -> Result<(), Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        spawn_blocking_with_tracing(move || {
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
        params: CreateIrrigationScheduleParams,
    ) -> Result<IrrigationSchedule, Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        let irrigation_sched = spawn_blocking_with_tracing(move || {
            let days = params
                .days_of_week
                .iter()
                .map(|d| d.to_string())
                .collect::<Vec<String>>()
                .join(",");

            let hoses = params
                .hoses
                .iter()
                .map(|d| d.to_string())
                .collect::<Vec<String>>()
                .join(",");

            diesel::insert_into(irrigation_schedule::table)
                .values((
                    irrigation_schedule_dsl::active.eq(params.active),
                    irrigation_schedule_dsl::name.eq(params.name),
                    irrigation_schedule_dsl::duration.eq(params.duration),
                    irrigation_schedule_dsl::start_time.eq(params.start_time),
                    irrigation_schedule_dsl::days_of_week.eq(days),
                    irrigation_schedule_dsl::hoses.eq(hoses),
                    //TODO: check created at
                ))
                .get_result::<IrrigationSchedule>(&mut conn)
                .map_err(|e| anyhow!("Error creating irrigation schedule: {}", e))
        })
        .await??;

        Ok(irrigation_sched)
    }

    async fn create_password_reset(&self, current_user: User) -> Result<Token, Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        let token_result = spawn_blocking_with_tracing(move || {
            let token = Token::new_password_reset(current_user.id);

            diesel::update(user::table)
                .filter(user::email.eq(current_user.email))
                .set((
                    user::password_reset_token.eq(token.value.clone()),
                    user::password_reset_token_expires_at.eq(token.expires_at),
                ))
                .execute(&mut conn)
                .map_err(|e| anyhow!(e))?;

            Ok::<Token, Error>(token)
        })
        .await??;

        Ok(token_result)
    }

    async fn create_sump_event(&self, info: String, kind: String) -> Result<(), Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        let _ = spawn_blocking_with_tracing(move || {
            diesel::insert_into(sump_event::table)
                .values((sump_event_dsl::info.eq(info), sump_event_dsl::kind.eq(kind)))
                .execute(&mut conn)
                .map_err(|e| anyhow!("Error creating sump event: {}", e))
        })
        .await?;

        Ok(())
    }

    async fn create_user(
        &self,
        new_email: String,
        new_password_hash: String,
        req_ip_address: String,
    ) -> Result<User, Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        let new_user: User = spawn_blocking_with_tracing(move || {
            conn.transaction::<_, Error, _>(|conn| {
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
            })
        })
        .await??;

        Ok(new_user)
    }

    async fn create_user_event(
        &self,
        user_for_event: &User,
        request_event_type: EventType,
        request_ip_address: String,
    ) -> Result<(), Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;
        let user_for_event = user_for_event.clone();

        let _new_user_event = spawn_blocking_with_tracing(move || {
            diesel::insert_into(user_event::table)
                .values((
                    user_event::user_id.eq(user_for_event.id),
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

    async fn delete_irrigation_schedule(&self, sched_id: i32) -> Result<Option<usize>, Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        let maybe_row_deleted = spawn_blocking_with_tracing(move || {
            match diesel::delete(irrigation_schedule::table)
                .filter(irrigation_schedule::id.eq(sched_id))
                .execute(&mut conn)
            {
                Ok(0) => Ok(None),
                Ok(n) => Ok(Some(n)),
                Err(e) => Err(e),
            }
        })
        .await
        .map_err(|e| anyhow!(e.to_string()))??;

        Ok(maybe_row_deleted)
    }

    async fn finish_irrigation_event(&self) -> Result<(), Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        let _row_updated = spawn_blocking_with_tracing(move || {
            let rows_updated = diesel::update(irrigation_event::table)
                .filter(irrigation_event::status.eq(IrrigationEventStatus::InProgress.to_string()))
                .set(irrigation_event::status.eq(IrrigationEventStatus::Completed.to_string()))
                .execute(&mut conn)
                .map_err(|e| anyhow!(e.to_string()))?;

            if rows_updated != 1 {
                tracing::error!("Expected to update 1 row, but updated {}", rows_updated);
            }

            Ok::<usize, Error>(rows_updated)
        })
        .await??;

        Ok(())
    }

    async fn irrigation_events(&self) -> Result<Vec<IrrigationEvent>, Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        let irrigation_events = spawn_blocking_with_tracing(move || {
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
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        let irrigation_schedules = spawn_blocking_with_tracing(move || {
            irrigation_schedule_dsl::irrigation_schedule
                .limit(100)
                .load::<IrrigationSchedule>(&mut conn)
                .map_err(|e| anyhow!(e))
        })
        .await??;

        Ok(irrigation_schedules)
    }

    async fn irrigation_schedule_by_id(&self, sched_id: i32) -> Result<IrrigationSchedule, Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        let irrigation_sched = spawn_blocking_with_tracing(move || {
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

    async fn next_queued_irrigation_event(
        &self,
    ) -> Result<Option<(IrrigationEvent, IrrigationSchedule)>, Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        let event = spawn_blocking_with_tracing(move || {
            let event = irrigation_event::table
                .inner_join(
                    irrigation_schedule::table
                        .on(irrigation_event::schedule_id.eq(irrigation_schedule::id)),
                )
                .filter(irrigation_event::status.eq(IrrigationEventStatus::Queued.to_string()))
                .order(irrigation_event::created_at.asc())
                .first::<(IrrigationEvent, IrrigationSchedule)>(&mut conn);

            match event {
                Ok(event) => Ok(Some(event)),
                Err(DieselError::NotFound) => Ok(None),
                Err(e) => Err(anyhow!(
                    "Internal server error when fetching queued event: {}",
                    e
                )),
            }
        })
        .await??;

        Ok(event)
    }

    async fn pool(&self) -> Result<Pool<ConnectionManager<SqliteConnection>>, Error> {
        Ok(self.pool.clone())
    }

    /// Creates events in 'queued' status for any schedules that are eligible to run.
    async fn queue_irrigation_events(
        &self,
        schedules: Vec<IrrigationSchedule>,
    ) -> Result<(), Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        let new_events: Vec<NewIrrigationEvent> = schedules
            .iter()
            .flat_map(|schedule| {
                schedule
                    .hoses
                    .split(',')
                    .filter_map(|hose| hose.parse::<i32>().ok())
                    .map(|hose_id| NewIrrigationEvent {
                        schedule_id: schedule.id,
                        hose_id,
                        status: IrrigationEventStatus::Queued.to_string(),
                        created_at: Utc::now().naive_utc(),
                        end_time: None,
                    })
                    .collect::<Vec<NewIrrigationEvent>>()
            })
            .collect();

        spawn_blocking_with_tracing(move || {
            diesel::insert_into(irrigation_event::table)
                .values(&new_events)
                .execute(&mut conn)
                .map_err(|e| anyhow!("Error creating irrigation events: {}", e))
        })
        .await?
        .map_err(|e| anyhow!("Internal server error when creating irrigation events: {e}"))?;

        Ok(())
    }

    // TODO: move Token from auth module
    async fn reset_password(
        &self,
        password: &Password,
        token: String,
    ) -> Result<(), ResetPasswordError> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| ResetPasswordError::DatabaseError(anyhow!(e)))?;

        let pw_hash = password
            .hash()
            .map_err(ResetPasswordError::InvalidPassword)?;

        let _row_updated = spawn_blocking_with_tracing(move || {
            let current_user = match user::table
                .filter(user::password_reset_token.eq(token))
                .first::<User>(&mut conn)
            {
                Ok(current_user) => current_user,
                Err(e) => match e {
                    DieselError::NotFound => return Err(ResetPasswordError::InvalidToken),
                    e => return Err(ResetPasswordError::DatabaseError(anyhow!(e))),
                },
            };

            if current_user.password_reset_token_expires_at.unwrap() < Utc::now().naive_utc() {
                return Err(ResetPasswordError::TokenExpired);
            }

            let result = diesel::update(user::table)
                .filter(user::email.eq(current_user.email))
                .set((
                    user::password_hash.eq(pw_hash),
                    user::password_reset_token.eq::<Option<String>>(None),
                    user::password_reset_token_expires_at.eq::<Option<String>>(None),
                ))
                .execute(&mut conn)
                .map_err(|e| ResetPasswordError::DatabaseError(anyhow!(e)))?;

            Ok::<usize, ResetPasswordError>(result)
        })
        .await
        .map_err(|e| ResetPasswordError::InternalServerError(anyhow!(e)))??;

        Ok(())
    }

    async fn schedule_statuses(&self) -> Result<Vec<ScheduleStatus>, Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        let statuses = spawn_blocking_with_tracing(move || {
            IrrigationEvent::status_query()
                .load::<StatusQueryResult>(&mut conn)
                .map(build_statuses)
                .map_err(|e| anyhow!(e))
        })
        .await?
        .map_err(|e| anyhow!(e))?;

        Ok(statuses)
    }

    async fn sump_events(&self) -> Result<Vec<SumpEvent>, Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        let sump_events = spawn_blocking_with_tracing(move || {
            sump_event_dsl::sump_event
                .limit(100)
                .load::<SumpEvent>(&mut conn)
                .map_err(|e| anyhow!(e))
        })
        .await??;

        Ok(sump_events)
    }

    async fn update_irrigation_schedule(
        &self,
        schedule_id: i32,
        params: UpdateIrrigationScheduleParams,
    ) -> Result<Option<IrrigationSchedule>, Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        let irrigation_sched = spawn_blocking_with_tracing(move || {
            let result = irrigation_schedule_dsl::irrigation_schedule
                .filter(irrigation_schedule_dsl::id.eq(schedule_id))
                .first::<IrrigationSchedule>(&mut conn);

            match result {
                Ok(mut irrigation_sched) => {
                    if let Some(active) = params.active {
                        irrigation_sched.active = active;
                    }
                    if let Some(name) = params.name {
                        irrigation_sched.name = name;
                    }
                    if let Some(duration) = params.duration {
                        irrigation_sched.duration = duration;
                    }
                    if let Some(start_time) = params.start_time {
                        irrigation_sched.start_time = start_time;
                    }
                    if let Some(days_of_week) = params.days_of_week {
                        irrigation_sched.days_of_week = days_of_week
                            .iter()
                            .map(|d| d.to_string())
                            .collect::<Vec<String>>()
                            .join(",");
                    }
                    if let Some(hoses) = params.hoses {
                        irrigation_sched.hoses = hoses
                            .iter()
                            .map(|d| d.to_string())
                            .collect::<Vec<String>>()
                            .join(",");
                    }

                    let irrigation_sched_clone = irrigation_sched.clone();

                    let _row_updated = diesel::update(irrigation_schedule::table)
                        .filter(irrigation_schedule::id.eq(schedule_id))
                        .set((
                            irrigation_schedule::active.eq(irrigation_sched.active),
                            irrigation_schedule::name.eq(irrigation_sched.name),
                            irrigation_schedule::duration.eq(irrigation_sched.duration),
                            irrigation_schedule::start_time.eq(irrigation_sched.start_time),
                            irrigation_schedule::days_of_week.eq(irrigation_sched.days_of_week),
                            irrigation_schedule::hoses.eq(irrigation_sched.hoses),
                        ))
                        .execute(&mut conn)
                        .map_err(|e| anyhow!(e))?;

                    Ok(Some(irrigation_sched_clone))
                }
                Err(e) => match e {
                    DieselError::NotFound => Ok(None),
                    _ => Err(anyhow!(e)),
                },
            }
        })
        .await??;

        Ok(irrigation_sched)
    }

    async fn update_user(&self, updates: UserUpdateFilter) -> Result<(), Error> {
        let pool = self.pool.clone();

        let result = spawn_blocking_with_tracing(move || {
            let mut conn = pool
                .get()
                .map_err(|e| anyhow!("Error getting database connection: {:?}", e))?;

            conn.transaction::<_, diesel::result::Error, _>(|conn| {
                // Look up the user
                let user = user::table
                    .filter(user::id.eq(updates.id))
                    .first::<User>(conn)?;

                // Apply the updates
                diesel::update(&user).set(&updates).execute(conn)?;

                Ok(())
            })
            .map_err(|e| anyhow!("Could not update user: {:?}", e))
        })
        .await?
        .map_err(|e| anyhow!("Error while updating user: {:?}", e))?;

        Ok(result)
    }

    async fn user_events(
        &self,
        user_id: i32,
        event_type: Option<EventType>,
        count: i64,
    ) -> Result<Vec<UserEvent>, Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        let user_events = spawn_blocking_with_tracing(move || {
            let mut event_filter: BoxedSelectStatement<_, _, _, _> = user_event::table.into_boxed();

            if let Some(event_type) = event_type {
                let filter: Box<dyn BoxableExpression<user_event::table, _, SqlType = Bool>> =
                    Box::new(user_event::event_type.eq(event_type.to_string()));
                event_filter = event_filter.filter(filter);
            }

            event_filter
                .filter(user_event::user_id.eq(user_id))
                .order(user_event::created_at.desc())
                .limit(count)
                .load::<UserEvent>(&mut conn)
                .map_err(|e| anyhow!(e.to_string()))
        })
        .await??;

        Ok(user_events)
    }

    async fn users(&self, filter: UserFilter) -> Result<Vec<User>, Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        let users = spawn_blocking_with_tracing(move || {
            let mut user_filter: BoxedSelectStatement<_, _, _, _> = user::table.into_boxed();

            if let Some(email) = filter.email {
                let filter: Box<dyn BoxableExpression<user::table, _, SqlType = Bool>> =
                    Box::new(user::email.eq(email));
                user_filter = user_filter.filter(filter);
            }

            if let Some(email_verif_token) = filter.email_verification_token {
                let filter: Box<dyn BoxableExpression<user::table, _, SqlType = Nullable<Bool>>> =
                    Box::new(user::email_verification_token.eq(email_verif_token));
                user_filter = user_filter.filter(filter);
            }

            user_filter
                .order(user::created_at.desc())
                .limit(100)
                .load::<User>(&mut conn)
                .map_err(|e| anyhow!(e.to_string()))
        })
        .await??;

        Ok(users)
    }

    async fn verify_email(&self, token: String) -> Result<(), VerifyEmailError> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| VerifyEmailError::DatabaseError(anyhow!(e)))?;

        let _result = spawn_blocking_with_tracing(move || {
            let user: User = user::table
                .filter(user::email_verification_token.eq(Some(token.clone())))
                .first::<User>(&mut conn)
                .map_err(|e| match e {
                    DieselError::NotFound => VerifyEmailError::EmailNotFound,
                    e => VerifyEmailError::DatabaseError(anyhow!(e)),
                })?;

            if user.email_verified_at.is_some() {
                return Err(VerifyEmailError::EmailAlreadyVerified);
            }

            // TODO: Reverse the comparison so it's not on a String, also remove unwrap
            if user.email_verification_token_expires_at.unwrap() < Utc::now().naive_utc() {
                return Err(VerifyEmailError::TokenExpired);
            }

            let _row_update_count = diesel::update(user::table)
                .filter(user::email_verification_token.eq(token))
                .set((
                    user::email_verification_token.eq(None::<String>),
                    user::email_verification_token_expires_at.eq(None::<String>),
                    user::email_verified_at.eq(Utc::now().naive_utc()),
                ))
                .execute(&mut conn)
                .map_err(|e| VerifyEmailError::DatabaseError(anyhow!(e)))?;

            Ok(())
        })
        .await
        .map_err(|e| VerifyEmailError::InternalServerError(anyhow!(e)))??;

        Ok(())
    }
}

fn build_statuses(results: Vec<StatusQueryResult>) -> Vec<ScheduleStatus> {
    results
        .into_iter()
        .map(|result: StatusQueryResult| {
            let StatusQueryResult {
                id,
                active,
                name,
                duration,
                start_time,
                days_of_week,
                hoses,
                created_at,
                updated_at,
                event_id,
                hose_id,
                status,
                end_time,
                event_created_at,
            } = result;

            let schedule = IrrigationSchedule {
                id,
                active,
                name,
                duration,
                start_time: NaiveTime::parse_from_str(&start_time, "%H:%M:%S%.9f").unwrap(),
                days_of_week,
                hoses,
                created_at: NaiveDateTime::parse_from_str(&created_at, "%Y-%m-%d %H:%M:%S")
                    .unwrap(),
                updated_at: NaiveDateTime::parse_from_str(&updated_at, "%Y-%m-%d %H:%M:%S")
                    .unwrap(),
            };

            if event_id.is_none() {
                return ScheduleStatus {
                    schedule,
                    last_event: None,
                };
            }

            if event_id.is_none()
                || hose_id.is_none()
                || status.is_none()
                || event_created_at.is_none()
            {
                return ScheduleStatus {
                    schedule,
                    last_event: None,
                };
            }

            let end_time = match end_time {
                Some(et) => match NaiveDateTime::parse_from_str(&et, "%Y-%m-%d %H:%M:%S") {
                    Ok(et) => Some(et),
                    Err(e) => {
                        tracing::error!("Error parsing end time: {:?}", e);
                        return ScheduleStatus {
                            schedule,
                            last_event: None,
                        };
                    }
                },
                None => None,
            };

            let last_event = IrrigationEvent {
                id: event_id.unwrap(),
                hose_id: hose_id.unwrap(),
                schedule_id: id,
                status: status.unwrap(),
                end_time,
                created_at: NaiveDateTime::parse_from_str(
                    &event_created_at.unwrap(),
                    "%Y-%m-%d %H:%M:%S%.9f",
                )
                .unwrap(),
            };

            ScheduleStatus {
                schedule,
                last_event: Some(last_event),
            }
        })
        .collect::<Vec<ScheduleStatus>>()
}
// mod tests {
//     use chrono::NaiveDateTime;
//     use rstest::rstest;

//     use crate::{
//         hydro::schedule::ScheduleStatus,
//         repository::{
//             implementation::build_statuses,
//             models::{
//                 irrigation_event::{IrrigationEvent, StatusQueryResult},
//                 irrigation_schedule::IrrigationSchedule,
//             },
//         },
//         test_fixtures::{
//             irrigation::{
//                 event::completed_event,
//                 schedule::{daily_schedule, tues_thurs_schedule},
//                 status::all_schedules_statuses,
//                 status_query::status_query_results,
//             },
//             tests::time,
//         },
//     };

//     #[rstest]
//     fn build_statuses_success(
//         #[from(status_query_results)] status_query_results: Vec<StatusQueryResult>,
//         #[from(completed_event)] completed_event: IrrigationEvent,
//         #[from(daily_schedule)] daily_schedule: IrrigationSchedule,
//         #[from(tues_thurs_schedule)] tues_thurs_schedule: IrrigationSchedule,
//     ) {
//         let due = build_statuses(status_query_results);
//         assert!(
//             due == vec![
//                 ScheduleStatus {
//                     schedule: daily_schedule,
//                     last_event: Some(completed_event.clone())
//                 },
//                 ScheduleStatus {
//                     schedule: tues_thurs_schedule,
//                     last_event: Some(completed_event)
//                 },
//             ]
//         )
//     }

//     // "Test Schedule Friday" has an event that has run already; the others
//     // are deactivated or not run on Fridays. This leaves two schedules due.
//     #[rstest]
//     fn due_statuses_success(
//         #[from(all_schedules_statuses)] statuses: Vec<ScheduleStatus>,
//         #[from(time)] time: NaiveDateTime,
//     ) {
//         let due = due_statuses(statuses, time);

//         assert!(due.len() == 2);
//         assert!(due
//             .iter()
//             .find(|s| s.schedule.name == "Test Schedule Daily")
//             .is_some());
//         assert!(due
//             .iter()
//             .find(|s| s.schedule.name == "Test Schedule Weekday")
//             .is_some());
//     }
// }
