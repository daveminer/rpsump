use serde::{Deserialize, Serialize};

pub mod auth;
pub mod info;
pub mod sump_event;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    pub message: String,
}

impl ApiResponse {
    pub fn bad_request(message: String) -> actix_web::HttpResponse {
        actix_web::HttpResponse::BadRequest().json(Self { message })
    }

    pub fn internal_server_error() -> actix_web::HttpResponse {
        actix_web::HttpResponse::InternalServerError().json(Self {
            message: "Internal server error".to_string(),
        })
    }

    pub fn ok(message: String) -> actix_web::HttpResponse {
        actix_web::HttpResponse::Ok().json(Self { message })
    }

    pub fn unauthorized(message: String) -> actix_web::HttpResponse {
        actix_web::HttpResponse::Unauthorized().json(Self { message })
    }
}
