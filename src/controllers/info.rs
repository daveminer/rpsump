use actix_web::{get, HttpRequest, HttpResponse, Responder};
use actix_web::{web::Data, Result};

use crate::auth::authenticated_user::AuthenticatedUser;
use crate::controllers::ApiResponse;
use crate::hydro::Hydro;

#[get("/info")]
async fn info(req: HttpRequest, _user: AuthenticatedUser) -> Result<impl Responder> {
    let sump_arc = req.app_data::<Data<Option<Hydro>>>();
    let maybe_sump = if sump_arc.is_none() {
        return Ok(ApiResponse::ok("Sump disabled.".to_string()));
    } else {
        // If shared object is present then it should be populated
        sump_arc.unwrap().as_ref()
    };

    let _sump = match maybe_sump {
        Some(sump) => sump,
        None => {
            tracing::error!(
                target = module_path!(),
                "Sump AppData was found but Sump was None"
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

    Ok(HttpResponse::Ok().json("r#{}"))
    //Ok(HttpResponse::Ok().json(&sensor_state))
}
