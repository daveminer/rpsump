use actix_web::{web, web::Data};
use anyhow::{anyhow, Error};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::AsExpression;
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;

use crate::database::DbPool;
use crate::models::user::User;
use crate::schema::user_event;
use crate::schema::user_event::dsl::*;

#[derive(Clone, PartialEq, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = user_event)]
pub struct UserEvent {
    pub id: i32,
    pub user_id: i32,
    pub event_type: String,
    pub ip_address: String,
    pub created_at: NaiveDateTime,
}

#[derive(Clone, Debug, DbEnum, AsExpression, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[diesel(sql_type = diesel::sql_types::Text)]
pub enum EventType {
    FailedLogin,
    LockedLogin,
    Login,
    Logout,
    PasswordReset,
    Signup,
}

impl Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::FailedLogin => write!(f, "failed_login"),
            EventType::LockedLogin => write!(f, "locked_login"),
            EventType::Login => write!(f, "login"),
            EventType::Logout => write!(f, "logout"),
            EventType::PasswordReset => write!(f, "password_reset"),
            EventType::Signup => write!(f, "signup"),
        }
    }
}

impl UserEvent {
    #[tracing::instrument(skip(request_user, db))]
    pub async fn check_allowed_status(
        request_user: Option<User>,
        ip_addr: String,
        event_limit: i64,
        db: Data<DbPool>,
    ) -> Result<(), Error> {
        // Parameterize this
        let time_limit = chrono::Utc::now() - chrono::Duration::hours(24);

        let events: Result<Vec<UserEvent>, Error> = web::block(move || {
            let mut conn = db.get()?;
            let mut query = user_event::table
                //.filter(user_event::user_id.eq(request_user.id))
                .filter(user_event::ip_address.eq(&ip_addr))
                // TODO: test timestamp
                .filter(user_event::created_at.gt(time_limit.naive_utc().to_string()))
                .into_boxed();

            if request_user.is_some() {
                query = query.filter(user_event::user_id.eq(request_user.unwrap().id));
            }

            query
                .order(user_event::created_at.desc())
                .limit(event_limit)
                .get_results::<UserEvent>(&mut conn)
                .map_err(|e| e.into())
        })
        .await?;

        let mut event_map = HashMap::new();

        // TODO: fix this option handling
        for event in events.unwrap() {
            let event_clone = event.clone();
            let events_for_type = event_map.entry(event.event_type).or_insert(Vec::new());
            events_for_type.push(event_clone);
        }

        let events_at_limit: Vec<(String, Vec<UserEvent>)> = event_map
            .into_iter()
            .filter(|(_, events)| events.len() >= 3)
            .collect();

        // TODO: refactor
        if events_at_limit.len() > 0 {
            let mut event_types = Vec::new();
            for (event_name, _) in events_at_limit {
                event_types.push(event_name);
            }

            return Err(anyhow!(
                "Too many events for event types: {:?}",
                event_types
            ));
        }

        Ok(())
    }

    #[tracing::instrument(skip(request_user, db))]
    pub async fn recent_events(
        request_user: Option<User>,
        ip_addr: Option<String>,
        event: EventType,
        count: i64,
        db: Data<DbPool>,
    ) -> Result<Vec<UserEvent>, Error> {
        if request_user.is_none() && ip_addr.is_none() {
            return Err(anyhow!("Must provide either a user or ip address."));
        }

        let mut query = user_event
            .filter(event_type.eq(event.to_string()))
            .into_boxed();

        if request_user.is_some() {
            query = query.filter(user_id.eq(request_user.unwrap().id));
        }

        if ip_addr.is_some() {
            query = query.filter(ip_address.eq(ip_addr.unwrap()));
        }

        web::block(move || {
            let mut conn = db.get().expect("Could not get a db connection.");

            query.limit(count).load(&mut conn)
        })
        .await?
        .map_err(|_| anyhow!("Internal server error when getting recent user signups."))
    }

    #[tracing::instrument(skip(request_user, db))]
    pub async fn create(
        request_user: User,
        request_ip_address: String,
        user_event_type: EventType,
        db: Data<DbPool>,
    ) -> Result<UserEvent, Error> {
        let new_user_event = web::block(move || {
            let mut conn = db.get().expect("Could not get a db connection.");

            diesel::insert_into(user_event::table)
                .values((
                    user_id.eq(request_user.id),
                    //event_type.eq(user_event_type.to_string()),
                    ip_address.eq(request_ip_address),
                ))
                .get_result(&mut conn)
        })
        .await?
        .map_err(|_| anyhow!("Internal server error when creating user event."))?;

        Ok(new_user_event)
    }
}
