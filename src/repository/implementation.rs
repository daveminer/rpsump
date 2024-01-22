use std::collections::HashMap;

use anyhow::{anyhow, Error};
use chrono::Utc;
use diesel::query_dsl::methods::LimitDsl;
use diesel::r2d2::{self, ConnectionManager, Pool, PooledConnection};
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::sqlite::SqliteConnection;

use crate::hydro::schedule::Status;
use crate::models::irrigation_event::{IrrigationEvent, IrrigationEventStatus, StatusQueryResult};
use crate::models::irrigation_schedule::IrrigationSchedule;
use crate::models::sump_event::SumpEvent;
use crate::models::user::User;
use crate::models::user_event::{EventType, UserEvent};
use crate::schema::irrigation_event::dsl::{hose_id, irrigation_event, schedule_id, status};
use crate::schema::irrigation_schedule::dsl::{id as irrigation_schedule_id, irrigation_schedule};
use crate::schema::sump_event::dsl::*;
use crate::schema::user_event::dsl::*;
use crate::schema::{user, user_event};
use crate::util::spawn_blocking_with_tracing;

pub type DbConn = PooledConnection<ConnectionManager<SqliteConnection>>;
type DbPool = Pool<ConnectionManager<SqliteConnection>>;

pub struct Implementation {
    pool: DbPool,
}

impl Implementation {
    pub fn new(database_uri: String) -> Result<Self, r2d2::Error> {
        let manager = ConnectionManager::<SqliteConnection>::new(database_uri);
        let pool = Pool::new(manager)?;

        Ok(Implementation { pool })
    }

    async fn create_irrigation_event(
        &self,
        schedule: IrrigationSchedule,
        hose: String,
    ) -> Result<IrrigationEvent, Error> {
        spawn_blocking_with_tracing(move || {
            let mut conn = self
                .pool
                .get()
                .map_err(|e| format!("Database error: {:?}", e))?;

            diesel::insert_into(irrigation_event::table)
                .values((
                    schedule_id.eq(schedule.id),
                    hose_id.eq(hose),
                    status.eq(IrrigationEventStatus::Queued.to_string()),
                ))
                .execute(&mut conn)
                .map_err(|e| anyhow!("Error creating irrigation event: {}", e))
        })
        .await?
        .map_err(|e| anyhow!("Internal server error when creating irrigation event: {e}"))?;

        Ok(())
    }

    fn create_irrigation_schedule(
        &self,
        // TODO: make type
        params: HashMap<String, String>,
    ) -> Result<IrrigationSchedule, Error> {
        let irrigation_sched = spawn_blocking_with_tracing(move || {
            let mut conn = self
                .pool
                .get()
                .map_err(|e| format!("Database error: {:?}", e))?;

            diesel::insert_into(irrigation_schedule::table)
                .values((
                    irrigation_schedule::name.eq(params.get("name").unwrap()),
                    irrigation_schedule::hoses.eq(params.get("hoses").unwrap()),
                    irrigation_schedule::start_time.eq(params.get("start_time").unwrap()),
                    irrigation_schedule::end_time.eq(params.get("end_time").unwrap()),
                    irrigation_schedule::duration.eq(params.get("duration").unwrap()),
                    irrigation_schedule::interval.eq(params.get("interval").unwrap()),
                    irrigation_schedule::interval_unit.eq(params.get("interval_unit").unwrap()),
                    irrigation_schedule::enabled.eq(params.get("enabled").unwrap()),
                ))
                .execute(&mut conn)
                .map_err(|e| anyhow!("Error creating irrigation schedule: {}", e))
        });

        Ok(irrigation_sched)
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
            let user = self.transaction::<_, Error, _>(|conn| {
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
        request_user_id: i32,
        request_event_type: String,
        request_ip_address: String,
    ) -> Result<UserEvent, Error> {
        let new_user_event = spawn_blocking_with_tracing(move || {
            let mut conn = self
                .pool
                .get()
                .map_err(|e| format!("Database error: {:?}", e))?;

            diesel::insert_into(user_event::table)
                .values((
                    user_id.eq(request_user_id),
                    event_type.eq(request_event_type.to_string()),
                    ip_address.eq(request_ip_address),
                ))
                .execute(&mut conn)
        })
        .await?
        .map_err(|e| anyhow!("Internal server error when creating user event: {}", e))?;

        Ok(new_user_event)
    }
    async fn delete_irrigation_schedule(
        &self,
        schedule_id: i32,
    ) -> Result<IrrigationSchedule, Error> {
    }

    async fn irrigation_events(&self) -> Result<Vec<IrrigationEvent>, Error> {
        let irrigation_events = spawn_blocking_with_tracing(move || {
            let mut conn = self
                .pool
                .get()
                .map_err(|e| format!("Database error: {:?}", e))?;

            irrigation_event
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
                .map_err(|e| format!("Database error: {:?}", e))?;

            irrigation_schedule
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
                .map_err(|e| format!("Database error: {:?}", e))?;

            irrigation_schedule
                .filter(irrigation_schedule_id.eq(sched_id))
                .first::<IrrigationSchedule>(&mut conn)
                .map_err(|e| match e {
                    DieselError::NotFound => anyhow!("Irrigation schedule not found."),
                    e => anyhow!(
                        "Internal server error when fetching irrigation schedule: {}",
                        e
                    ),
                })
        });

        Ok(irrigation_sched)
    }

    async fn schedule_statuses(&self) -> Result<Vec<Status>, Error> {
        let statuses = spawn_blocking_with_tracing(move || {
            let mut conn = self
                .pool
                .get()
                .map_err(|e| format!("Database error: {:?}", e))?;

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
                .map_err(|e| format!("Database error: {:?}", e))?;

            sump_event
                .limit(100)
                .load::<SumpEvent>(&mut conn)
                .map_err(|e| anyhow!(e))
        });

        Ok(sump_events)
    }
    // fn update_user(&self, user_id: i32, email: String) -> Result<User, Error>;
    // fn update_irrigation_event(&self, event_id: i32) -> Result<IrrigationEvent, Error>;
    // fn update_irrigation_schedule(&self, schedule_id: i32) -> Result<IrrigationSchedule, Error>;
    // fn user_events(&self, user_id: i32, count: i64) -> Result<Vec<UserEvent>, Error>;
    // fn users(&self) -> Result<Vec<User>, Error>;
    async fn user_by_email(&self, email: String) -> Result<User, Error> {
        let user = spawn_blocking_with_tracing(move || {
            let mut conn = self
                .pool
                .get()
                .map_err(|e| format!("Database error: {:?}", e))?;

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

    async fn verify_email(&self, token: String) -> Result<User, Error> {
        let _result = spawn_blocking_with_tracing(move || {
            let mut conn = self
                .pool
                .get()
                .map_err(|e| format!("Database error: {:?}", e))?;

            let user_from_token =
                match Self::by_email_verification_token(token.clone()).first::<User>(&mut conn) {
                    Ok(user) => user,
                    Err(DieselError::NotFound) => {
                        return Err(anyhow!("Invalid token."));
                    }
                    Err(e) => {
                        return Err(anyhow!("Internal server error when verifying email: {}", e));
                    }
                };

            if user_from_token.email_verified_at.is_some() {
                return Err(anyhow!("Email already verified."));
            }

            if let Err(e) = Self::check_email_verification_expiry(
                user_from_token.email_verification_token_expires_at,
            ) {
                return Err(e);
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
