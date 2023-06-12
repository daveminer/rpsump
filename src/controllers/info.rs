use actix_web::{get, HttpResponse, Responder};
use actix_web::{web::Data, Result};

use crate::auth::authenticated_user::AuthenticatedUser;
use crate::controllers::ApiResponse;
use crate::sump::Sump;

#[get("/info")]
async fn info(sump: Option<Data<Sump>>, _user: AuthenticatedUser) -> Result<impl Responder> {
    if sump.is_none() {
        return Ok(ApiResponse::ok("Sump disabled.".to_string()));
    }

    let sump = sump.unwrap();

    let sensor_state = match sump.sensor_state.lock() {
        Ok(sensor_state) => *sensor_state,
        Err(_) => {
            return Ok(HttpResponse::InternalServerError().body("Could not lock sensor state."));
        }
    };

    let body = match serde_json::to_string(&sensor_state) {
        Ok(body) => body,
        Err(_) => {
            return Ok(
                HttpResponse::InternalServerError().body("Could not serialize sensor state.")
            );
        }
    };

    Ok(HttpResponse::Ok().body(body))
}
