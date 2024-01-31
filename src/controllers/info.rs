use std::sync::{Arc, Mutex};

use actix_web::{get, HttpRequest, HttpResponse};
use actix_web::{web::Data, Result};
use serde_json::json;

use crate::auth::authenticated_user::AuthenticatedUser;

use crate::get_hydro;
use crate::hydro::Hydro;

#[get("/info")]
#[tracing::instrument(skip(maybe_hydro, _user))]
async fn info(
    _req: HttpRequest,
    _user: AuthenticatedUser,
    maybe_hydro: Data<Mutex<Option<Hydro>>>,
) -> Result<HttpResponse> {
    let hydro = get_hydro!(maybe_hydro);
    // let sensor_state = match sump.sensor_state.lock() {
    //     Ok(sensor_state) => *sensor_state,
    //     Err(e) => {
    //         tracing::error!(
    //             target = module_path!(),
    //             error = e.to_string(),
    //             "Could not get sensor state"
    //         );
    //         return Ok(ApiResponse::internal_server_error());
    //     }
    // };

    Ok(HttpResponse::Ok().json(json!({"heater": hydro.heater.is_on()})))
}
