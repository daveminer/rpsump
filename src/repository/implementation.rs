use anyhow::{anyhow, Error};
use async_trait::async_trait;
use chrono::{NaiveDateTime, NaiveTime, Utc};

use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::sqlite::SqliteConnection;
use diesel::{Connection, ExpressionMethods, QueryDsl, RunQueryDsl};

use crate::auth::password::Password;
use crate::auth::token::Token;
use crate::hydro::schedule::Status;
use crate::repository::models::{
    irrigation_event::{IrrigationEvent, IrrigationEventStatus, StatusQueryResult},
    irrigation_schedule::{
        CreateIrrigationScheduleParams, IrrigationSchedule, UpdateIrrigationScheduleParams,
    },
    sump_event::SumpEvent,
    user::User,
    user_event::EventType,
};
use crate::repository::Repository;
use crate::schema::{irrigation_event, irrigation_schedule, sump_event, user, user_event};
use crate::schema::{
    irrigation_event::dsl as irrigation_event_dsl,
    irrigation_schedule::dsl as irrigation_schedule_dsl, sump_event::dsl as sump_event_dsl,
};
use crate::util::spawn_blocking_with_tracing;

type DbPool = Pool<ConnectionManager<SqliteConnection>>;

pub struct Implementation {
    pool: DbPool,
}

impl Implementation {
    pub async fn new(database_uri: Option<String>) -> Result<Self, Error> {
        let connection_string = if database_uri.is_none() {
            ":memory:".to_string()
        } else {
            database_uri.unwrap()
        };

        let manager = ConnectionManager::<SqliteConnection>::new(connection_string);
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
        // TODO: make type
        params: CreateIrrigationScheduleParams,
    ) -> Result<(), Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        let _irrigation_sched = spawn_blocking_with_tracing(move || {
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
                .execute(&mut conn)
                .map_err(|e| anyhow!("Error creating irrigation schedule: {}", e))
        })
        .await??;

        Ok(())
    }

    async fn create_password_reset(&self, current_user: User) -> Result<(), Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        let _row_updated = spawn_blocking_with_tracing(move || {
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

    // TODO: implement
    async fn delete_irrigation_schedule(&self, _sched_id: i32) -> Result<(), Error> {
        Ok(())
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

    async fn next_queued_irrigation_event(&self) -> Result<Option<(i32, IrrigationEvent)>, Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        let result = spawn_blocking_with_tracing(move || {
            let result = irrigation_event::table
                .inner_join(irrigation_schedule::table)
                .filter(irrigation_event::status.eq(IrrigationEventStatus::Queued.to_string()))
                .select((irrigation_schedule::duration, irrigation_event::all_columns))
                .order(irrigation_event::created_at.asc())
                .first::<(i32, IrrigationEvent)>(&mut conn);

            match result {
                Ok(event) => Ok(Some(event)),
                Err(e) => match e {
                    DieselError::NotFound => Ok(None),
                    _ => Err(anyhow!(e)),
                },
            }
        })
        .await??;

        Ok(result)
    }

    async fn new(path: Option<String>) -> Result<Self, Error> {
        let path = if path.is_some() {
            path.unwrap()
        } else {
            ":memory:".to_string()
        };

        let manager = ConnectionManager::<SqliteConnection>::new(path);
        let pool = Pool::new(manager)?;

        Ok(Implementation { pool })
    }

    // TODO: finish
    async fn queue_irrigation_events(&self, _events: Vec<Status>) -> Result<(), Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        let _: () = spawn_blocking_with_tracing(move || {
            let _row_updated = diesel::update(irrigation_event::table)
                .filter(irrigation_event::status.eq(IrrigationEventStatus::Queued.to_string()))
                .set(irrigation_event::status.eq(IrrigationEventStatus::InProgress.to_string()))
                .execute(&mut conn)
                .map_err(|e| anyhow!(e))?;

            Ok::<(), Error>(())
        })
        .await??;

        Ok(())
    }

    // TODO: move Token from auth module
    async fn reset_password(&self, password: &Password, token: String) -> Result<(), Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        let pw_hash = password.hash()?;

        let _row_updated = spawn_blocking_with_tracing(move || {
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

    async fn schedule_statuses(&self) -> Result<Vec<Status>, Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        let statuses = spawn_blocking_with_tracing(move || {
            IrrigationEvent::status_query()
                .load::<StatusQueryResult>(&mut conn)
                .map(|results| build_statuses(results))
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
    // fn update_user(&self, user_id: i32, email: String) -> Result<User, Error>;
    // fn update_irrigation_event(&self, event_id: i32) -> Result<IrrigationEvent, Error>;
    async fn update_irrigation_schedule(
        &self,
        schedule_id: i32,
        params: UpdateIrrigationScheduleParams,
    ) -> Result<Option<IrrigationSchedule>, Error> {
        let mut conn = self
            .clone()
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
            // .map_err(|e| match e {
            //     DieselError::NotFound => None,
            //     e => Err(anyhow!(
            //         "Internal server error when fetching irrigation schedule: {}",
            //         e
            //     )),
            // })?;

            //Ok(result)
        })
        .await??;

        Ok(irrigation_sched)
    }
    // fn user_events(&self, user_id: i32, count: i64) -> Result<Vec<UserEvent>, Error>;
    // fn users(&self) -> Result<Vec<User>, Error>;
    async fn user_by_email(&self, email: String) -> Result<Option<User>, Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        spawn_blocking_with_tracing(move || {
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

    // TODO: review
    async fn validate_login(&self, email: String, _password: String) -> Result<User, Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        let user = spawn_blocking_with_tracing(move || {
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
        let mut conn = self
            .pool
            .get()
            .map_err(|e| anyhow!("Database error: {:?}", e))?;

        let _result = spawn_blocking_with_tracing(move || {
            let user: User = user::table
                .filter(user::email_verification_token.eq(token.clone()))
                .first::<User>(&mut conn)
                .map_err(|e| match e {
                    DieselError::NotFound => anyhow!("Invalid token."),
                    e => anyhow!("Internal server error when verifying email: {}", e),
                })?;

            if user.email_verified_at.is_some() {
                return Err(anyhow!("Email already verified."));
            }

            // TODO: Reverse the comparison so it's not on a String, also remove unwrap
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

fn build_statuses(results: Vec<StatusQueryResult>) -> Vec<Status> {
    let statuses = results
        .into_iter()
        .map(|result: StatusQueryResult| {
            let StatusQueryResult {
                schedule_schedule_id,
                schedule_active,
                schedule_name,
                schedule_duration,
                schedule_start_time,
                schedule_days_of_week,
                schedule_hoses,
                schedule_created_at,
                schedule_updated_at,
                event_id,
                event_hose_id,
                event_status,
                event_created_at,
                event_end_time,
                ..
            } = result;

            let schedule = IrrigationSchedule {
                id: schedule_schedule_id,
                active: schedule_active,
                name: schedule_name,
                duration: schedule_duration,
                start_time: NaiveTime::parse_from_str(&schedule_start_time, "%H:%M:%S").unwrap(),
                days_of_week: schedule_days_of_week,
                hoses: schedule_hoses,
                created_at: NaiveDateTime::parse_from_str(
                    &schedule_created_at,
                    "%Y-%m-%d %H:%M:%S",
                )
                .unwrap(),
                updated_at: NaiveDateTime::parse_from_str(
                    &schedule_updated_at,
                    "%Y-%m-%d %H:%M:%S",
                )
                .unwrap(),
            };

            if event_id.is_none() {
                return Status {
                    schedule,
                    last_event: None,
                };
            }

            let last_event = IrrigationEvent {
                id: event_id.unwrap(),
                hose_id: event_hose_id.unwrap(),
                schedule_id: schedule_schedule_id,
                status: event_status.unwrap(),
                end_time: Some(
                    NaiveDateTime::parse_from_str(&event_end_time.unwrap(), "%Y-%m-%d %H:%M:%S")
                        .unwrap(),
                ),
                created_at: NaiveDateTime::parse_from_str(
                    &event_created_at.unwrap(),
                    "%Y-%m-%d %H:%M:%S",
                )
                .unwrap(),
            };

            Status {
                schedule,
                last_event: Some(last_event),
            }
        })
        .collect::<Vec<Status>>();

    statuses
}
