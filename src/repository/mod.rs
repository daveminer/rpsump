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

use crate::auth::password::Password;
use crate::hydro::schedule::Status;
use crate::repository::models::{
    irrigation_schedule::CreateIrrigationScheduleParams,
    user::{UserFilter, UserUpdateFilter},
};

/// Used in the application to access the database
pub type Repo = &'static dyn Repository;

/// Exposes the pool for data setup in integration tests
pub type TestRepo = &'static dyn TestRepository;

/// Creates a testable interface for the database pool.
#[automock]
#[async_trait]
pub trait Repository: Send + Sync + 'static {
    async fn create_irrigation_event(
        &self,
        schedule: IrrigationSchedule,
        hose: i32,
    ) -> Result<(), Error>;
    async fn create_irrigation_schedule(
        &self,
        params: CreateIrrigationScheduleParams,
    ) -> Result<(), Error>;
    // TODO: move this import out of controllers module
    async fn create_password_reset(&self, user: User) -> Result<(), Error>;
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
    async fn delete_irrigation_schedule(&self, sched_id: i32) -> Result<(), Error>;
    async fn finish_irrigation_event(&self) -> Result<(), Error>;
    async fn irrigation_events(&self) -> Result<Vec<IrrigationEvent>, Error>;
    async fn irrigation_schedules(&self) -> Result<Vec<IrrigationSchedule>, Error>;
    async fn irrigation_schedule_by_id(&self, sched_id: i32) -> Result<IrrigationSchedule, Error>;
    async fn next_queued_irrigation_event(&self) -> Result<Option<(i32, IrrigationEvent)>, Error>;
    async fn new(path: Option<String>) -> Result<Self, Error>
    where
        Self: Sized;
    async fn pool(&self) -> Result<Pool<ConnectionManager<SqliteConnection>>, Error>;
    async fn queue_irrigation_events(&self, events: Vec<Status>) -> Result<(), Error>;
    async fn reset_password(&self, new_password: &Password, token: String) -> Result<(), Error>;
    async fn schedule_statuses(&self) -> Result<Vec<Status>, Error>;
    //fn update_user(&self, user_id: i32, email: String) -> Result<User, Error>;
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
    async fn verify_email(&self, token: String) -> Result<(), Error>;
}

#[async_trait]
pub trait TestRepository: Repository {
    async fn pool(&self) -> Result<Pool<ConnectionManager<SqliteConnection>>, Error>;
    // Other test-specific methods
}

pub async fn implementation(database_uri: Option<String>) -> Result<Repo, Error> {
    let implementation = implementation::Implementation::new(database_uri).await?;
    let repository = Box::new(implementation);

    Ok(Box::leak(repository))
}

pub async fn mock() -> Repo {
    let repository = Box::new(
        MockRepository::new(None)
            .await
            .expect("Could not create mock repository."),
    );

    Box::leak(repository)
}
