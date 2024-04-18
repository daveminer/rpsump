use anyhow::anyhow;
use std::sync::Mutex;

use actix_web::{
    post,
    web::{self, Data},
    HttpResponse, Result,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    auth::authenticated_user::AuthenticatedUser, controllers::auth::helpers::error_response,
    hydro::Hydro,
};

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
#[tracing::instrument(skip(_user, hydro))]
pub async fn heater(
    params: web::Json<HeaterParams>,
    _user: AuthenticatedUser,
    hydro: Data<Mutex<Hydro>>,
) -> Result<HttpResponse> {
    let mut hydro = match hydro.lock() {
        Ok(lock) => lock,
        Err(e) => {
            return Ok(error_response(
                anyhow!(e.to_string()),
                "Could not get hydro lock",
            ));
        }
    };

    match params.switch {
        HeaterLevel::On => {
            println!("Turning heater on");
            if let Err(e) = hydro.heater.on().await {
                tracing::error!("Error while turning heater on: {:?}", e);
                return Ok(HttpResponse::InternalServerError().body(e.to_string()));
            }
        }
        HeaterLevel::Off => {
            println!("Turning heater off");
            if let Err(e) = hydro.heater.off().await {
                tracing::error!("Error while turning heater off: {:?}", e);
                return Ok(HttpResponse::InternalServerError().body(e.to_string()));
            }
        }
    };

    tracing::info!("Heater status changed: {:?}", params.switch);

    Ok(HttpResponse::Ok().json(json!({"status":"ok"})))
}
