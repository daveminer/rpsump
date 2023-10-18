use actix_web::rt::task::{spawn_blocking, JoinHandle};
use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};

pub mod auth;
pub mod info;
pub mod irrigation;
pub mod sump_event;

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

/// Convenience method for database connections during an api request.
#[macro_export]
macro_rules! new_conn {
    ($db:expr) => {
        match $db.get() {
            Ok(conn) => conn,
            Err(e) => {
                tracing::error!("Could not get database connection: {}", e);
                return Ok(ApiResponse::internal_server_error());
            }
        }
    };
}
