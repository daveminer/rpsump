use std::sync::Mutex;

use actix_web::{get, HttpRequest, HttpResponse};
use actix_web::{web::Data, Result};
use serde_json::json;

use crate::auth::authenticated_user::AuthenticatedUser;

use crate::hydro::Hydro;

#[get("/info")]
#[tracing::instrument(skip(maybe_hydro, _user))]
async fn info(
    _req: HttpRequest,
    _user: AuthenticatedUser,
    maybe_hydro: Data<Mutex<Option<Hydro>>>,
) -> Result<HttpResponse> {
    let lock = match maybe_hydro.lock() {
        Ok(lock) => lock,
        Err(e) => {
            tracing::error!(
                target = module_path!(),
                error = e.to_string(),
                "Could not get lock on hydro"
            );
            return Ok(HttpResponse::InternalServerError().finish());
        }
    };

    let hydro = match *lock {
        Some(ref hydro) => hydro,
        None => {
            tracing::error!(target = module_path!(), "Hydro not initialized");
            return Ok(disabled_response());
        }
    };

    Ok(HttpResponse::Ok().json(json!({
        "heater": hydro.heater.is_on().await,
        "poolPumpSpeed": hydro.pool_pump.speed().await,
    })))
}

fn disabled_response() -> HttpResponse {
    HttpResponse::BadRequest().json(json!({
        "message": "disabled"
    }))
}
