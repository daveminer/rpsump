pub mod implementation;
pub mod models;

use anyhow::Error;
use async_trait::async_trait;
use chrono::NaiveDateTime;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use mockall::automock;
use models::{
    garden_event::{GardenEvent, GardenEventFilter, GardenEventStatus},
    garden_schedule::{
        CreateGardenScheduleParams, GardenSchedule, UpdateGardenScheduleParams,
    },
    invite::Invite,
    sump_event::SumpEvent,
    user::User,
    user_event::{EventType, UserEvent},
};

use crate::auth::{password::Password, token::Token};
use crate::hydro::weather::PrecipSnapshot;
use crate::repository::models::user::{UserFilter, UserUpdateFilter};

use self::implementation::{RefreshTokenError, ResetPasswordError, VerifyEmailError};

/// Used in the application to access the database
pub type Repo = &'static dyn Repository;

/// Creates a testable interface for the database.
#[automock]
#[async_trait]
pub trait Repository: Send + Sync + 'static {
    /// Moves a queued event to in-progress. Returns `false` when the event is
    /// no longer queued (cancelled or already started), in which case it must
    /// not be run.
    async fn begin_garden_event(&self, event_id: i32) -> Result<bool, Error>;
    async fn consume_refresh_token(&self, token_value: String) -> Result<i32, RefreshTokenError>;
    async fn create(path: Option<String>) -> Result<Self, Error>
    where
        Self: Sized;
    async fn create_email_verification(&self, user: &User) -> Result<Token, Error>;
    async fn create_invite(&self, email: String, invited_by_user_id: i32)
        -> Result<Invite, Error>;
    async fn create_garden_schedule(
        &self,
        params: CreateGardenScheduleParams,
    ) -> Result<GardenSchedule, Error>;
    /// Queues a one-off run. Returns `None` when another event is already
    /// queued or in progress, so repeated taps of "Start" cannot stack up.
    async fn create_manual_garden_event(
        &self,
        duration_secs: i32,
    ) -> Result<Option<GardenEvent>, Error>;
    async fn create_password_reset(&self, user: User) -> Result<Token, Error>;
    async fn create_refresh_token(&self, token: &Token) -> Result<(), Error>;
    async fn create_sump_event(&self, info: String, kind: String) -> Result<(), Error>;
    async fn create_user(
        &self,
        new_email: String,
        new_password_hash: String,
        req_ip_address: String,
    ) -> Result<User, Error>;
    async fn create_user_event(
        &self,
        user: &User,
        request_event_type: EventType,
        request_ip_address: String,
    ) -> Result<(), Error>;
    async fn current_garden_event(&self) -> Result<Option<GardenEvent>, Error>;
    async fn delete_garden_schedule(&self, sched_id: i32) -> Result<Option<usize>, Error>;
    async fn finish_garden_event(
        &self,
        event_id: i32,
        status: GardenEventStatus,
    ) -> Result<(), Error>;
    async fn garden_event_by_id(&self, event_id: i32) -> Result<Option<GardenEvent>, Error>;
    async fn garden_events(
        &self,
        filter: GardenEventFilter,
    ) -> Result<Vec<GardenEvent>, Error>;
    async fn garden_schedule_by_id(
        &self,
        sched_id: i32,
    ) -> Result<Option<GardenSchedule>, Error>;
    async fn garden_schedules(&self) -> Result<Vec<GardenSchedule>, Error>;
    async fn invite_by_token(&self, token: String) -> Result<Option<Invite>, Error>;
    async fn next_queued_garden_event(&self) -> Result<Option<GardenEvent>, Error>;
    async fn pool(&self) -> Result<Pool<ConnectionManager<SqliteConnection>>, Error>;
    async fn queue_due_garden_events(
        &self,
        now: NaiveDateTime,
        precip: &PrecipSnapshot,
    ) -> Result<usize, Error>;
    /// Cancels the in-progress event and anything already queued behind it.
    /// Returns the ids that were cancelled, newest first.
    async fn redeem_invite(&self, invite_id: i32, accepted_by_user_id: i32) -> Result<(), Error>;
    async fn request_garden_stop(&self) -> Result<Vec<i32>, Error>;
    async fn revoke_refresh_tokens_for_user(&self, user_id: i32) -> Result<(), Error>;
    async fn reset_password(
        &self,
        password: &Password,
        token: String,
    ) -> Result<(), ResetPasswordError>;
    async fn sump_events(&self) -> Result<Vec<SumpEvent>, Error>;
    async fn update_garden_schedule(
        &self,
        sched_id: i32,
        params: UpdateGardenScheduleParams,
    ) -> Result<Option<GardenSchedule>, Error>;
    async fn user_events(
        &self,
        user_id: i32,
        event_type: Option<EventType>,
        count: i64,
    ) -> Result<Vec<UserEvent>, Error>;
    async fn update_user(&self, filter: UserUpdateFilter) -> Result<(), Error>;
    async fn users(&self, filter: UserFilter) -> Result<Vec<User>, Error>;
    async fn verify_email(&self, token: String) -> Result<(), VerifyEmailError>;
}

/// Migrations are compiled into the binary so a deployed artifact always
/// carries the schema it expects. Applying them at startup removes the
/// separate `diesel migration run` step, which is easy to omit during a
/// deploy and fails silently: the table is simply absent and every query
/// against it returns a 500.
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// Applies any migrations the database has not yet seen. Returns the versions
/// that were applied, so a deploy can be seen in the logs rather than inferred.
pub fn run_pending_migrations(conn: &mut SqliteConnection) -> Result<Vec<String>, Error> {
    let applied = conn
        .run_pending_migrations(MIGRATIONS)
        .map_err(|e| anyhow::anyhow!("Could not run pending migrations: {e}"))?;

    Ok(applied.iter().map(|v| v.to_string()).collect())
}

pub async fn implementation(database_uri: Option<String>) -> Result<Repo, Error> {
    let implementation = implementation::Implementation::create(database_uri).await?;
    let repository = Box::new(implementation);

    Ok(Box::leak(repository))
}

#[cfg(test)]
mod migration_tests {
    use super::*;
    use diesel::{Connection, RunQueryDsl};

    #[test]
    fn migrations_apply_to_a_fresh_database() {
        let mut conn = SqliteConnection::establish(":memory:").unwrap();

        let applied = run_pending_migrations(&mut conn).expect("migrations should apply");
        assert!(!applied.is_empty(), "a fresh database should need migrations");

        // The absence of this table is what returned 500s from /auth/signup
        // after #68 was deployed without running migrations.
        diesel::sql_query("SELECT id FROM invite LIMIT 1")
            .execute(&mut conn)
            .expect("invite table should exist once migrations have run");
    }

    #[test]
    fn running_twice_applies_nothing_the_second_time() {
        let mut conn = SqliteConnection::establish(":memory:").unwrap();

        run_pending_migrations(&mut conn).expect("first run should apply migrations");
        let second = run_pending_migrations(&mut conn).expect("second run should succeed");

        // Startup must be idempotent: the service restarts routinely and
        // systemd is configured Restart=always.
        assert!(
            second.is_empty(),
            "re-running should be a no-op, applied: {second:?}"
        );
    }
}
