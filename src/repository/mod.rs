mod implementation;

use anyhow::Error;
use async_trait::async_trait;
use mockall::automock;
use std::collections::HashMap;

use crate::auth::token::Token;

use crate::models::{
    irrigation_event::IrrigationEvent,
    irrigation_schedule::IrrigationSchedule,
    sump_event::SumpEvent,
    user::User,
    user_event::{EventType, UserEvent},
};

/// Creates a testable interface for the database pool.
#[automock]
#[async_trait]
pub trait Repository: Send + Sync {
    fn create_irrigation_event(
        &self,
        schedule_id: i32,
        event_type: String,
        ip_address: String,
    ) -> Result<IrrigationEvent, Error>;
    fn create_irrigation_schedule(
        &self,
        params: HashMap<String, String>,
    ) -> Result<IrrigationSchedule, Error>;
    fn create_sump_event(
        &self,
        sump_id: i32,
        event_type: String,
        ip_address: String,
    ) -> Result<SumpEvent, Error>;
    fn create_user(&self, email: String, password_hash: String) -> Result<User, Error>;
    async fn create_user_event(
        &self,
        user: User,
        event_type: EventType,
        ip_address: String,
    ) -> Result<UserEvent, Error>;
    async fn delete_irrigation_schedule(&self, sched_id: i32) -> Result<IrrigationSchedule, Error>;
    async fn irrigation_events(&self) -> Result<Vec<IrrigationEvent>, Error>;
    async fn irrigation_schedules(&self) -> Result<Vec<IrrigationSchedule>, Error>;
    async fn irrigation_schedule_by_id(&self, sched_id: i32) -> Result<IrrigationSchedule, Error>;
    fn password_reset(&self, user: User, token: Token) -> Result<User, Error>;
    async fn schedule_statuses(&self) -> Result<Vec<IrrigationSchedule>, Error>;
    fn update_user(&self, user_id: i32, email: String) -> Result<User, Error>;
    async fn sump_events() -> Result<Vec<SumpEvent>, Error>;
    fn update_irrigation_event(&self, event_id: i32) -> Result<IrrigationEvent, Error>;
    fn update_irrigation_schedule(
        &self,
        sched_id: i32,
        params: HashMap<String, String>,
    ) -> Result<IrrigationSchedule, Error>;
    fn user_events(&self, user_id: i32, count: i64) -> Result<Vec<UserEvent>, Error>;
    fn users(&self) -> Result<Vec<User>, Error>;
    async fn user_by_email(&self, email: String) -> Result<Option<User>, Error>;
    async fn validate_login(&self, email: String, password: String) -> Result<User, Error>;
    async fn verify_email(&self, token: String) -> Result<User, Error>;
}

/// Used in the application to access the database
pub type Repo = &'static dyn Repository;

pub fn implementation(database_uri: String) -> Result<Repo, Error> {
    let repository = Box::new(implementation::Implementation::new(database_uri)?);

    Ok(Box::leak(repository))
}
