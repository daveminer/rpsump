use std::sync::Mutex;

use actix_web::{get, HttpRequest, HttpResponse};
use actix_web::{web::Data, Result};
use serde_json::json;

use crate::auth::authenticated_user::AuthenticatedUser;
use crate::hydro::Hydro;

#[get("/info")]
#[tracing::instrument(skip(hydro, _user))]
async fn info(
    _req: HttpRequest,
    _user: AuthenticatedUser,
    hydro: Data<Mutex<Hydro>>,
) -> Result<HttpResponse> {
    let hydro = match hydro.lock() {
        Ok(hydro) => hydro,
        Err(e) => {
            tracing::error!(
                target = module_path!(),
                error = e.to_string(),
                "Could not get hydro"
            );
            return Ok(HttpResponse::InternalServerError().finish());
        }
    };

    Ok(HttpResponse::Ok().json(json!({
        "heater": hydro.heater.is_on().await,
        "poolPumpSpeed": hydro.pool_pump.speed().await,
    })))
}
