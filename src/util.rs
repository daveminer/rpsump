use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};
use tokio::task::{spawn_blocking, JoinHandle};

pub const BAD_CREDS: &str = "Invalid email or password.";
pub const PASSWORD_LOWER: &str = "Password must contain a lowercase letter.";
pub const PASSWORD_NUMBER: &str = "Password must contain a number.";
pub const PASSWORD_SPECIAL: &str = "Password must contain a special character.";
pub const PASSWORD_TOO_SHORT: &str = "Password is too short.";
pub const PASSWORD_TOO_LONG: &str = "Password is too long.";
pub const PASSWORD_UPPER: &str = "Password must contain an uppercase letter.";
pub const REQUIRED_FIELDS: &str = "Email and password are required.";

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

    pub fn not_found() -> HttpResponse {
        HttpResponse::NotFound().json(Self {
            message: "Not found".to_string(),
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
