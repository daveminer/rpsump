pub mod implementation;
pub mod models;

use anyhow::Error;
use async_trait::async_trait;
use chrono::NaiveDateTime;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
use mockall::automock;
use models::{
    garden_event::{GardenEvent, GardenEventSource, GardenEventStatus},
    garden_schedule::{
        CreateGardenScheduleParams, GardenSchedule, UpdateGardenScheduleParams,
    },
    sump_event::SumpEvent,
    user::User,
    user_event::{EventType, UserEvent},
};

use crate::auth::{password::Password, token::Token};
use crate::repository::models::user::{UserFilter, UserUpdateFilter};

use self::implementation::{RefreshTokenError, ResetPasswordError, VerifyEmailError};

/// Used in the application to access the database
pub type Repo = &'static dyn Repository;

/// Creates a testable interface for the database.
#[automock]
#[async_trait]
pub trait Repository: Send + Sync + 'static {
    async fn begin_garden_event(&self, event_id: i32) -> Result<(), Error>;
    async fn consume_refresh_token(&self, token_value: String) -> Result<i32, RefreshTokenError>;
    async fn create(path: Option<String>) -> Result<Self, Error>
    where
        Self: Sized;
    async fn create_email_verification(&self, user: &User) -> Result<Token, Error>;
    async fn create_garden_schedule(
        &self,
        params: CreateGardenScheduleParams,
    ) -> Result<GardenSchedule, Error>;
    async fn create_manual_garden_event(
        &self,
        duration_secs: i32,
    ) -> Result<GardenEvent, Error>;
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
        limit: i64,
        offset: i64,
        source: Option<GardenEventSource>,
    ) -> Result<Vec<GardenEvent>, Error>;
    async fn garden_schedule_by_id(
        &self,
        sched_id: i32,
    ) -> Result<Option<GardenSchedule>, Error>;
    async fn garden_schedules(&self) -> Result<Vec<GardenSchedule>, Error>;
    async fn next_queued_garden_event(&self) -> Result<Option<GardenEvent>, Error>;
    async fn pool(&self) -> Result<Pool<ConnectionManager<SqliteConnection>>, Error>;
    async fn queue_due_garden_events(&self, now: NaiveDateTime) -> Result<usize, Error>;
    async fn request_garden_stop(&self) -> Result<Option<i32>, Error>;
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

pub async fn implementation(database_uri: Option<String>) -> Result<Repo, Error> {
    let implementation = implementation::Implementation::create(database_uri).await?;
    let repository = Box::new(implementation);

    Ok(Box::leak(repository))
}
