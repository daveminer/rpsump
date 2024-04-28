pub mod implementation;
pub mod models;

use anyhow::Error;
use async_trait::async_trait;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
use mockall::automock;
use models::{
    irrigation_event::IrrigationEvent,
    irrigation_schedule::{IrrigationSchedule, UpdateIrrigationScheduleParams},
    sump_event::SumpEvent,
    user::User,
    user_event::{EventType, UserEvent},
};

use crate::auth::{password::Password, token::Token};
use crate::hydro::schedule::ScheduleStatus;
use crate::repository::models::{
    irrigation_schedule::CreateIrrigationScheduleParams,
    user::{UserFilter, UserUpdateFilter},
};

use self::implementation::{ResetPasswordError, VerifyEmailError};

/// Used in the application to access the database
pub type Repo = &'static dyn Repository;

/// Creates a testable interface for the database.
#[automock]
#[async_trait]
pub trait Repository: Send + Sync + 'static {
    async fn begin_irrigation(&self, event: IrrigationEvent) -> Result<(), Error>;
    async fn create(path: Option<String>) -> Result<Self, Error>
    where
        Self: Sized;
    async fn create_email_verification(&self, user: &User) -> Result<Token, Error>;
    async fn create_irrigation_event(
        &self,
        schedule: IrrigationSchedule,
        hose: i32,
    ) -> Result<(), Error>;
    async fn create_irrigation_schedule(
        &self,
        params: CreateIrrigationScheduleParams,
    ) -> Result<IrrigationSchedule, Error>;
    async fn create_password_reset(&self, user: User) -> Result<Token, Error>;
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
    async fn delete_irrigation_schedule(&self, sched_id: i32) -> Result<Option<usize>, Error>;
    async fn finish_irrigation_event(&self) -> Result<(), Error>;
    async fn irrigation_events(&self) -> Result<Vec<IrrigationEvent>, Error>;
    async fn irrigation_schedules(&self) -> Result<Vec<IrrigationSchedule>, Error>;
    async fn irrigation_schedule_by_id(&self, sched_id: i32) -> Result<IrrigationSchedule, Error>;
    async fn next_queued_irrigation_event(
        &self,
    ) -> Result<Option<(IrrigationEvent, IrrigationSchedule)>, Error>;
    async fn pool(&self) -> Result<Pool<ConnectionManager<SqliteConnection>>, Error>;
    async fn queue_irrigation_events(
        &self,
        schedules: Vec<IrrigationSchedule>,
    ) -> Result<(), Error>;
    async fn reset_password(
        &self,
        password: &Password,
        token: String,
    ) -> Result<(), ResetPasswordError>;
    async fn schedule_statuses(&self) -> Result<Vec<ScheduleStatus>, Error>;
    async fn sump_events(&self) -> Result<Vec<SumpEvent>, Error>;
    async fn update_irrigation_schedule(
        &self,
        sched_id: i32,
        params: UpdateIrrigationScheduleParams,
    ) -> Result<Option<IrrigationSchedule>, Error>;
    async fn user_events(
        &self,
        user_id: i32,
        event_type: Option<EventType>,
        count: i64,
    ) -> Result<Vec<UserEvent>, Error>;
    async fn update_user(&self, filter: UserUpdateFilter) -> Result<(), Error>;
    async fn users(&self, filter: UserFilter) -> Result<Vec<User>, Error>;
    async fn validate_login(&self, email: String, password: String) -> Result<User, Error>;
    async fn verify_email(&self, token: String) -> Result<(), VerifyEmailError>;
}

pub async fn implementation(database_uri: Option<String>) -> Result<Repo, Error> {
    let implementation = implementation::Implementation::create(database_uri).await?;
    let repository = Box::new(implementation);

    Ok(Box::leak(repository))
}
