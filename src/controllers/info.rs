use actix_web::{get, Responder};
use actix_web::{web::Data, Result};

use crate::auth::authenticated_user::AuthenticatedUser;
use crate::controllers::ApiResponse;
use crate::sump::Sump;

#[get("/info")]
async fn info(sump: Option<Data<Sump>>, _user: AuthenticatedUser) -> Result<impl Responder> {
    let sump = if sump.is_none() {
        return Ok(ApiResponse::ok("Sump disabled.".to_string()));
    } else {
        sump.unwrap()
    };

    let sensor_state = match sump.sensor_state.lock() {
        Ok(sensor_state) => *sensor_state,
        Err(e) => {
            tracing::error!("Could not get sensor state: {}", e);
            return Ok(ApiResponse::internal_server_error());
        }
    };

    let body = match serde_json::to_string(&sensor_state) {
        Ok(body) => body,
        Err(e) => {
            tracing::error!("Could not serialize sensor state: {}", e);
            return Ok(ApiResponse::internal_server_error());
        }
    };

    Ok(ApiResponse::ok(body))
}
