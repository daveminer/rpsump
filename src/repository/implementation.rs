use anyhow::{anyhow, Error};
use async_trait::async_trait;
use chrono::{Datelike, NaiveDate, Utc, Weekday};

use crate::auth::password::Password;
use crate::auth::token::Token;
use crate::hydro::weather::{should_skip, PrecipSnapshot};
use crate::repository::models::{
    garden_event::{GardenEvent, GardenEventSource, GardenEventStatus, NewGardenEvent},
    garden_schedule::{
        days_to_csv, times_to_csv, CreateGardenScheduleParams, GardenSchedule,
        NewGardenSchedule, UpdateGardenScheduleParams,
    },
    sump_event::SumpEvent,
    user::User,
    user::UserFilter,
    user_event::{EventType, UserEvent},
};
use crate::repository::Repository;
use crate::schema::{
    garden_event, garden_schedule, refresh_token, sump_event, user, user_event,
};
use crate::schema::{
    garden_event::dsl as garden_event_dsl, garden_schedule::dsl as garden_schedule_dsl,
    sump_event::dsl as sump_event_dsl,
};
use crate::util::spawn_blocking_with_tracing;
use diesel::internal::table_macro::BoxedSelectStatement;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::sql_types::{Bool, Nullable};
use diesel::sqlite::SqliteConnection;
use diesel::BoxableExpression;
use diesel::{BoolExpressionMethods, Connection, ExpressionMethods, QueryDsl, RunQueryDsl};

use super::models::refresh_token::RefreshToken as RefreshTokenModel;
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
pub enum RefreshTokenError {
    #[error("Database error.")]
    DatabaseError(anyhow::Error),
    #[error("Internal server error.")]
    InternalServerError(anyhow::Error),
    #[error("Invalid token.")]
    InvalidToken,
    #[error("Token expired.")]
    TokenExpired,
    #[error("Token revoked.")]
    TokenRevoked,
}

impl From<DieselError> for RefreshTokenError {
    fn from(e: DieselError) -> Self {
        RefreshTokenError::DatabaseError(anyhow!(e))
    }
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
    async fn begin_garden_event(&self, event_id: i32) -> Result<(), Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        spawn_blocking_with_tracing(move || {
            conn.transaction::<_, Error, _>(|conn| {
                // Guard: only one in-progress event at a time.
                let in_progress = garden_event::table
                    .filter(garden_event::status.eq(GardenEventStatus::InProgress.to_string()))
                    .first::<GardenEvent>(conn);

                match in_progress {
                    Ok(_) => return Err(anyhow!("A garden event is already in progress")),
                    Err(DieselError::NotFound) => (),
                    Err(e) => return Err(anyhow!("Error checking in-progress event: {}", e)),
                }

                diesel::update(garden_event::table)
                    .filter(garden_event::id.eq(event_id))
                    .filter(garden_event::status.eq(GardenEventStatus::Queued.to_string()))
                    .set((
                        garden_event::status.eq(GardenEventStatus::InProgress.to_string()),
                        garden_event::start_time.eq(Utc::now().naive_utc()),
                    ))
                    .execute(conn)
                    .map_err(|e| anyhow!("Error starting garden event: {}", e))?;

                Ok(())
            })
        })
        .await??;

        Ok(())
    }

    async fn consume_refresh_token(
        &self,
        token_value: String,
    ) -> Result<i32, RefreshTokenError> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| RefreshTokenError::DatabaseError(anyhow!(e)))?;

        let user_id = spawn_blocking_with_tracing(move || {
            conn.transaction::<_, RefreshTokenError, _>(|conn| {
                let record = refresh_token::table
                    .filter(refresh_token::token.eq(&token_value))
                    .first::<RefreshTokenModel>(conn)
                    .map_err(|e| match e {
                        DieselError::NotFound => RefreshTokenError::InvalidToken,
                        e => RefreshTokenError::DatabaseError(anyhow!(e)),
                    })?;

                if record.revoked_at.is_some() {
                    // Reuse of a revoked token signals potential theft.
                    // Revoke all tokens for this user as a precaution.
                    diesel::update(refresh_token::table)
                        .filter(
                            refresh_token::user_id
                                .eq(record.user_id)
                                .and(refresh_token::revoked_at.is_null()),
                        )
                        .set(refresh_token::revoked_at.eq(Some(Utc::now().naive_utc())))
                        .execute(conn)?;

                    return Err(RefreshTokenError::TokenRevoked);
                }

                if record.expires_at < Utc::now().naive_utc() {
                    return Err(RefreshTokenError::TokenExpired);
                }

                diesel::update(refresh_token::table)
                    .filter(refresh_token::id.eq(record.id))
                    .set(refresh_token::revoked_at.eq(Some(Utc::now().naive_utc())))
                    .execute(conn)?;

                Ok(record.user_id)
            })
        })
        .await
        .map_err(|e| RefreshTokenError::InternalServerError(anyhow!(e)))??;

        Ok(user_id)
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

    async fn create_garden_schedule(
        &self,
        params: CreateGardenScheduleParams,
    ) -> Result<GardenSchedule, Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        let sched = spawn_blocking_with_tracing(move || {
            let new = NewGardenSchedule {
                name: params.name,
                active: params.active,
                start_times: times_to_csv(&params.start_times),
                days_of_week: days_to_csv(&params.days_of_week),
                duration_secs: params.duration_secs,
                skip_on_rain: params.skip_on_rain,
            };

            diesel::insert_into(garden_schedule::table)
                .values(&new)
                .get_result::<GardenSchedule>(&mut conn)
                .map_err(|e| anyhow!("Error creating garden schedule: {}", e))
        })
        .await??;

        Ok(sched)
    }

    async fn create_manual_garden_event(
        &self,
        duration_secs: i32,
    ) -> Result<GardenEvent, Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        let event = spawn_blocking_with_tracing(move || {
            let new = NewGardenEvent {
                schedule_id: None,
                source: GardenEventSource::Manual.to_string(),
                status: GardenEventStatus::Queued.to_string(),
                scheduled_for: Utc::now().naive_utc(),
                duration_secs,
            };

            diesel::insert_into(garden_event::table)
                .values(&new)
                .get_result::<GardenEvent>(&mut conn)
                .map_err(|e| anyhow!("Error creating manual garden event: {}", e))
        })
        .await??;

        Ok(event)
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

    async fn create_refresh_token(&self, token: &Token) -> Result<(), Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        let token_value = token.value.clone();
        let user_id = token.user_id;
        let expires_at = token.expires_at;

        spawn_blocking_with_tracing(move || {
            diesel::insert_into(refresh_token::table)
                .values((
                    refresh_token::user_id.eq(user_id),
                    refresh_token::token.eq(token_value),
                    refresh_token::expires_at.eq(expires_at),
                ))
                .execute(&mut conn)
                .map_err(|e| anyhow!("Error creating refresh token: {}", e))
        })
        .await?
        .map_err(|e| anyhow!("Internal server error when creating refresh token: {e}"))?;

        Ok(())
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

    async fn current_garden_event(&self) -> Result<Option<GardenEvent>, Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        let event = spawn_blocking_with_tracing(move || {
            match garden_event_dsl::garden_event
                .filter(garden_event_dsl::status.eq(GardenEventStatus::InProgress.to_string()))
                .order(garden_event_dsl::start_time.desc())
                .first::<GardenEvent>(&mut conn)
            {
                Ok(e) => Ok(Some(e)),
                Err(DieselError::NotFound) => Ok(None),
                Err(e) => Err(anyhow!("Error fetching current garden event: {}", e)),
            }
        })
        .await??;

        Ok(event)
    }

    async fn delete_garden_schedule(&self, sched_id: i32) -> Result<Option<usize>, Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        let maybe_row_deleted = spawn_blocking_with_tracing(move || {
            match diesel::delete(garden_schedule::table)
                .filter(garden_schedule::id.eq(sched_id))
                .execute(&mut conn)
            {
                Ok(0) => Ok(None),
                Ok(n) => Ok(Some(n)),
                Err(e) => Err(anyhow!(e)),
            }
        })
        .await??;

        Ok(maybe_row_deleted)
    }

    async fn finish_garden_event(
        &self,
        event_id: i32,
        status: GardenEventStatus,
    ) -> Result<(), Error> {
        if !matches!(
            status,
            GardenEventStatus::Completed | GardenEventStatus::Cancelled
        ) {
            return Err(anyhow!(
                "finish_garden_event requires a terminal status (completed or cancelled)"
            ));
        }

        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        spawn_blocking_with_tracing(move || {
            let rows_updated = diesel::update(garden_event::table)
                .filter(garden_event::id.eq(event_id))
                .set((
                    garden_event::status.eq(status.to_string()),
                    garden_event::end_time.eq(Utc::now().naive_utc()),
                ))
                .execute(&mut conn)
                .map_err(|e| anyhow!(e))?;

            if rows_updated != 1 {
                tracing::warn!(
                    event_id,
                    rows_updated,
                    "finish_garden_event affected unexpected row count"
                );
            }

            Ok::<usize, Error>(rows_updated)
        })
        .await??;

        Ok(())
    }

    async fn garden_event_by_id(&self, event_id: i32) -> Result<Option<GardenEvent>, Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        let event = spawn_blocking_with_tracing(move || {
            match garden_event_dsl::garden_event
                .filter(garden_event_dsl::id.eq(event_id))
                .first::<GardenEvent>(&mut conn)
            {
                Ok(e) => Ok(Some(e)),
                Err(DieselError::NotFound) => Ok(None),
                Err(e) => Err(anyhow!(e)),
            }
        })
        .await??;

        Ok(event)
    }

    async fn garden_events(
        &self,
        limit: i64,
        offset: i64,
        source: Option<GardenEventSource>,
    ) -> Result<Vec<GardenEvent>, Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        let events = spawn_blocking_with_tracing(move || {
            let mut q = garden_event_dsl::garden_event.into_boxed();
            if let Some(src) = source {
                q = q.filter(garden_event_dsl::source.eq(src.to_string()));
            }

            q.order(garden_event_dsl::created_at.desc())
                .limit(limit)
                .offset(offset)
                .load::<GardenEvent>(&mut conn)
                .map_err(|e| anyhow!(e))
        })
        .await??;

        Ok(events)
    }

    async fn garden_schedule_by_id(
        &self,
        sched_id: i32,
    ) -> Result<Option<GardenSchedule>, Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        let sched = spawn_blocking_with_tracing(move || {
            match garden_schedule_dsl::garden_schedule
                .filter(garden_schedule_dsl::id.eq(sched_id))
                .first::<GardenSchedule>(&mut conn)
            {
                Ok(s) => Ok(Some(s)),
                Err(DieselError::NotFound) => Ok(None),
                Err(e) => Err(anyhow!(e)),
            }
        })
        .await??;

        Ok(sched)
    }

    async fn garden_schedules(&self) -> Result<Vec<GardenSchedule>, Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        let schedules = spawn_blocking_with_tracing(move || {
            garden_schedule_dsl::garden_schedule
                .order(garden_schedule_dsl::created_at.desc())
                .limit(200)
                .load::<GardenSchedule>(&mut conn)
                .map_err(|e| anyhow!(e))
        })
        .await??;

        Ok(schedules)
    }

    async fn next_queued_garden_event(&self) -> Result<Option<GardenEvent>, Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        let event = spawn_blocking_with_tracing(move || {
            match garden_event_dsl::garden_event
                .filter(garden_event_dsl::status.eq(GardenEventStatus::Queued.to_string()))
                .order(garden_event_dsl::scheduled_for.asc())
                .first::<GardenEvent>(&mut conn)
            {
                Ok(e) => Ok(Some(e)),
                Err(DieselError::NotFound) => Ok(None),
                Err(e) => Err(anyhow!(e)),
            }
        })
        .await??;

        Ok(event)
    }

    async fn pool(&self) -> Result<Pool<ConnectionManager<SqliteConnection>>, Error> {
        Ok(self.pool.clone())
    }

    /// Walks every active schedule, computes its candidate `scheduled_for`
    /// instants for today that are at or before `now`, and inserts a queued
    /// event for any combo that doesn't already have one. The
    /// `(schedule_id, scheduled_for)` unique index makes the inserts
    /// idempotent across restarts.
    async fn queue_due_garden_events(
        &self,
        now: chrono::NaiveDateTime,
        precip: &PrecipSnapshot,
    ) -> Result<usize, Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        // Copy out of &PrecipSnapshot so the spawn_blocking closure can be 'static.
        let precip = *precip;

        let inserted = spawn_blocking_with_tracing(move || {
            let schedules: Vec<GardenSchedule> = garden_schedule_dsl::garden_schedule
                .filter(garden_schedule_dsl::active.eq(true))
                .load::<GardenSchedule>(&mut conn)
                .map_err(|e| anyhow!(e))?;

            let today_weekday: Weekday = now.date().weekday();
            let today: NaiveDate = now.date();

            let mut new_rows: Vec<NewGardenEvent> = Vec::new();
            for schedule in &schedules {
                let days = schedule.parsed_days_of_week();
                if !days.iter().any(|d| *d == today_weekday) {
                    continue;
                }
                let skip = should_skip(schedule, &precip);
                let status = if skip {
                    GardenEventStatus::Skipped
                } else {
                    GardenEventStatus::Queued
                };
                for time in schedule.parsed_start_times() {
                    let scheduled_for = today.and_time(time);
                    if scheduled_for > now {
                        continue;
                    }
                    if skip {
                        tracing::info!(
                            schedule_id = schedule.id,
                            name = %schedule.name,
                            past_mm = precip.past_mm,
                            forecast_mm = precip.forecast_mm,
                            current_mm = precip.current_mm,
                            threshold_mm = precip.threshold_mm,
                            "garden: skipping due event for precipitation"
                        );
                    }
                    new_rows.push(NewGardenEvent {
                        schedule_id: Some(schedule.id),
                        source: GardenEventSource::Scheduled.to_string(),
                        status: status.to_string(),
                        scheduled_for,
                        duration_secs: schedule.duration_secs,
                    });
                }
            }

            if new_rows.is_empty() {
                return Ok(0_usize);
            }

            // Insert one-at-a-time so a duplicate (caught by the unique index)
            // doesn't blow away the whole batch.
            let mut count = 0_usize;
            for row in new_rows {
                match diesel::insert_into(garden_event::table)
                    .values(&row)
                    .execute(&mut conn)
                {
                    Ok(n) => count += n,
                    Err(DieselError::DatabaseError(
                        DatabaseErrorKind::UniqueViolation,
                        _,
                    )) => {} // already queued
                    Err(e) => return Err(anyhow!(e)),
                }
            }
            Ok(count)
        })
        .await??;

        Ok(inserted)
    }

    async fn request_garden_stop(&self) -> Result<Option<i32>, Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        let maybe_id = spawn_blocking_with_tracing(move || {
            conn.transaction::<_, Error, _>(|conn| {
                let current = garden_event::table
                    .filter(
                        garden_event::status.eq(GardenEventStatus::InProgress.to_string()),
                    )
                    .first::<GardenEvent>(conn);

                match current {
                    Ok(event) => {
                        diesel::update(garden_event::table)
                            .filter(garden_event::id.eq(event.id))
                            .set(garden_event::status.eq(GardenEventStatus::Cancelled.to_string()))
                            .execute(conn)
                            .map_err(|e| anyhow!(e))?;
                        Ok(Some(event.id))
                    }
                    Err(DieselError::NotFound) => Ok(None),
                    Err(e) => Err(anyhow!(e)),
                }
            })
        })
        .await??;

        Ok(maybe_id)
    }

    async fn revoke_refresh_tokens_for_user(&self, user_id: i32) -> Result<(), Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        spawn_blocking_with_tracing(move || {
            diesel::update(refresh_token::table)
                .filter(
                    refresh_token::user_id
                        .eq(user_id)
                        .and(refresh_token::revoked_at.is_null()),
                )
                .set(refresh_token::revoked_at.eq(Some(Utc::now().naive_utc())))
                .execute(&mut conn)
                .map_err(|e| anyhow!("Error revoking refresh tokens: {}", e))
        })
        .await?
        .map_err(|e| anyhow!("Internal server error when revoking refresh tokens: {e}"))?;

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

    async fn update_garden_schedule(
        &self,
        schedule_id: i32,
        params: UpdateGardenScheduleParams,
    ) -> Result<Option<GardenSchedule>, Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        let sched = spawn_blocking_with_tracing(move || {
            let result = garden_schedule_dsl::garden_schedule
                .filter(garden_schedule_dsl::id.eq(schedule_id))
                .first::<GardenSchedule>(&mut conn);

            match result {
                Ok(mut s) => {
                    if let Some(name) = params.name {
                        s.name = name;
                    }
                    if let Some(active) = params.active {
                        s.active = active;
                    }
                    if let Some(start_times) = params.start_times {
                        s.start_times = times_to_csv(&start_times);
                    }
                    if let Some(days_of_week) = params.days_of_week {
                        s.days_of_week = days_to_csv(&days_of_week);
                    }
                    if let Some(duration_secs) = params.duration_secs {
                        s.duration_secs = duration_secs;
                    }
                    if let Some(skip_on_rain) = params.skip_on_rain {
                        s.skip_on_rain = skip_on_rain;
                    }
                    s.updated_at = Utc::now().naive_utc();

                    let updated = diesel::update(garden_schedule::table)
                        .filter(garden_schedule::id.eq(schedule_id))
                        .set((
                            garden_schedule::name.eq(&s.name),
                            garden_schedule::active.eq(s.active),
                            garden_schedule::start_times.eq(&s.start_times),
                            garden_schedule::days_of_week.eq(&s.days_of_week),
                            garden_schedule::duration_secs.eq(s.duration_secs),
                            garden_schedule::skip_on_rain.eq(s.skip_on_rain),
                            garden_schedule::updated_at.eq(s.updated_at),
                        ))
                        .get_result::<GardenSchedule>(&mut conn)
                        .map_err(|e| anyhow!(e))?;

                    Ok(Some(updated))
                }
                Err(DieselError::NotFound) => Ok(None),
                Err(e) => Err(anyhow!(e)),
            }
        })
        .await??;

        Ok(sched)
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
