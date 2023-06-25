use actix_web::{get, HttpRequest, HttpResponse, Responder};
use actix_web::{web::Data, Result};
use std::sync::Arc;

use crate::auth::authenticated_user::AuthenticatedUser;
use crate::controllers::ApiResponse;
use crate::sump::{Sump, TestSump};

#[get("/info")]
//async fn info(tsump: Data<Option<TestSump>>) -> Result<impl Responder> {
async fn info(req: HttpRequest, _user: AuthenticatedUser) -> Result<impl Responder> {
    let tsump = req.app_data::<Data<Option<TestSump>>>();
    if tsump.is_some() {
        println!("TSUMP : {:?}", tsump);
    } else {
        println!("TSUMP is None")
    }
    //let sump_arc = Arc::clone(&sump);
    //let sump_ref = sump_arc.as_ref();

    // let sump_obj = if sump_ref.is_none() {
    //     return Ok(ApiResponse::ok("Sump disabled.".to_string()));
    // } else {
    //     return Ok(ApiResponse::ok("".to_string()));
    //     //sump_ref.clone().unwrap()
    // };

    // let sensor_state = match sump_obj.sensor_state.lock() {
    //     Ok(sensor_state) => *sensor_state,
    //     Err(e) => {
    //         tracing::error!("Could not get sensor state: {}", e);
    //         return Ok(ApiResponse::internal_server_error());
    //     }
    // };

    // let body = match serde_json::to_string(&sensor_state) {
    //     Ok(body) => body,
    //     Err(e) => {
    //         tracing::error!("Could not serialize sensor state: {}", e);
    //         return Ok(ApiResponse::internal_server_error());
    //     }
    // };

    // Ok(ApiResponse::ok(body))
    Ok(ApiResponse::ok("".to_string()))
}
