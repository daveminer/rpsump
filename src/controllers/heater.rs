use std::sync::Mutex;

use actix_web::{
    post,
    web::{self, Data},
    HttpResponse, Responder, Result,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::auth::authenticated_user::AuthenticatedUser;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HeaterLevel {
    Off,
    On,
}

#[derive(Debug, Deserialize)]
pub struct HeaterParams {
    pub switch: HeaterLevel,
}

#[post("/heater")]
#[tracing::instrument(skip(_user))]
pub async fn heater(
    params: web::Json<HeaterParams>,
    _user: AuthenticatedUser,
) -> Result<HttpResponse> {
    // let mut lock = maybe_hydro.lock();
    // if lock.is_none() {
    //     return Ok(HttpResponse::Ok().body("Hydro not configured"));
    // }

    // let hydro = lock.as_mut().unwrap();
    // match params.switch {
    //     HeaterLevel::On => {
    //         println!("Turning heater on");
    //         if let Err(e) = hydro.heater.on().await {
    //             tracing::error!("Error while turning heater on: {:?}", e);
    //             return Ok(HttpResponse::InternalServerError().body(e.to_string()));
    //         }
    //     }
    //     HeaterLevel::Off => {
    //         println!("Turning heater off");
    //         if let Err(e) = hydro.heater.off().await {
    //             tracing::error!("Error while turning heater off: {:?}", e);
    //             return Ok(HttpResponse::InternalServerError().body(e.to_string()));
    //         }
    //     }
    // };

    // tracing::info!("Heater status changed: {:?}", params.switch);

    Ok(HttpResponse::Ok().json(json!({"status":"ok"})))
}
