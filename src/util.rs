use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};
use tokio::task::{spawn_blocking, JoinHandle};

pub const PASSWORD_LOWER: &str = "Password must contain a lowercase letter.";
pub const PASSWORD_NUMBER: &str = "Password must contain a number.";
pub const PASSWORD_SPECIAL: &str = "Password must contain a special character.";
pub const PASSWORD_TOO_SHORT: &str = "Password is too short.";
pub const PASSWORD_TOO_LONG: &str = "Password is too long.";
pub const PASSWORD_UPPER: &str = "Password must contain an uppercase letter.";

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    pub message: String,
}

/// Helper methods for api JSON responses that return no data.
impl ApiResponse {
    pub fn bad_request(message: String) -> HttpResponse {
        HttpResponse::BadRequest().json(Self { message })
    }

    pub fn internal_server_error() -> HttpResponse {
        HttpResponse::InternalServerError().json(Self {
            message: "Internal server error".to_string(),
        })
    }

    pub fn ok(message: String) -> HttpResponse {
        HttpResponse::Ok().json(Self { message })
    }

    pub fn unauthorized(message: String) -> HttpResponse {
        HttpResponse::Unauthorized().json(Self { message })
    }
}

/// Long-running calls to blocking functions need to be spawned on Actix's
/// blocking thread pool or the main event loop will be blocked.
/// This includes all calls to the database, as Diesel has a synchronous API.
pub fn spawn_blocking_with_tracing<F, R>(f: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let current_span = tracing::Span::current();
    spawn_blocking(move || current_span.in_scope(f))
}

#[macro_export]
macro_rules! get_hydro {
    ($maybe_hydro:expr) => {
        match $maybe_hydro.lock() {
            Ok(mut lock) => match lock.as_mut() {
                Some(hydro) => hydro.clone(),
                None => return Ok(HttpResponse::Ok().json(json!("Hydro not configured"))),
            },
            Err(e) => {
                let msg = "Cound not get hydro lock";
                tracing::error!(target = module_path!(), error = e.to_string(), msg);
                return Ok(HttpResponse::Ok().json(json!(msg)));
            }
        }
    };
}

/// Convenience method for database connections during an api request.
#[macro_export]
macro_rules! new_conn {
    ($db:expr) => {
        match $db.clone().get_conn() {
            Ok(conn) => conn,
            Err(e) => {
                tracing::error!(
                    target = module_path!(),
                    error = e.to_string(),
                    "Could not get database connection"
                );
                return Ok(ApiResponse::internal_server_error());
            }
        }
    };
}
