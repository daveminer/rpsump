use actix_web::{get, HttpRequest, HttpResponse, Responder};
use actix_web::{web::Data, Result};
use serde_json::json;

use crate::auth::authenticated_user::AuthenticatedUser;
use crate::controllers::ApiResponse;
use crate::hydro::Hydro;

#[get("/info")]
async fn info(req: HttpRequest, _user: AuthenticatedUser) -> Result<impl Responder> {
    let hydro_arc = req.app_data::<Data<Option<Hydro>>>();
    let maybe_hydro = if hydro_arc.is_none() {
        return Ok(ApiResponse::ok("Sump disabled.".to_string()));
    } else {
        // If shared object is present then it should be populated
        hydro_arc.unwrap().as_ref()
    };

    let hydro = match maybe_hydro {
        Some(hydro) => hydro,
        None => {
            tracing::error!(
                target = module_path!(),
                "Hydro AppData was found but Hydro was None"
            );
            return Ok(ApiResponse::internal_server_error());
        }
    };

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

    Ok(HttpResponse::Ok().json(json!({"heater": &hydro.heater.is_on()})))
}
