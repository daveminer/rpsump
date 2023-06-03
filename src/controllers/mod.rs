use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};

pub mod auth;
pub mod info;
pub mod sump_event;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    pub message: String,
}

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
