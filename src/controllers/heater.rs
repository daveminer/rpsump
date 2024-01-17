use actix_web::{
    post,
    web::{self, Data},
    HttpResponse, Responder, Result,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::hydro::Hydro;
use crate::{auth::authenticated_user::AuthenticatedUser, database::DbPool};

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
#[tracing::instrument(skip(_db, _user, maybe_hydro))]
pub async fn heater(
    params: web::Json<HeaterParams>,
    _db: Data<DbPool>,
    _user: AuthenticatedUser,
    maybe_hydro: web::Data<Option<Hydro>>,
) -> Result<impl Responder> {
    if maybe_hydro.is_none() {
        return Ok(HttpResponse::Ok().body("Hydro not configured."));
    }
    let mut hydro = maybe_hydro.as_ref().clone().unwrap();

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
