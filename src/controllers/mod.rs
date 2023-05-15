use serde::{Deserialize, Serialize};

pub mod auth;
pub mod info;
pub mod sump_event;

#[derive(Serialize, Deserialize)]
pub struct ErrorBody {
    pub reason: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
}
