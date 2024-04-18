use tokio::sync::Mutex;

use actix_web::{
    post,
    web::{self, Data},
    HttpResponse, Result,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{auth::authenticated_user::AuthenticatedUser, hydro::Hydro};

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
    let mut hydro = hydro.lock().await;

    match params.switch {
        HeaterLevel::On => {
            println!("Turning heater on");
            hydro.heater.on().await
        }
        HeaterLevel::Off => {
            println!("Turning heater off");
            hydro.heater.off().await
        }
    };

    tracing::info!("Heater status changed: {:?}", params.switch);

    Ok(HttpResponse::Ok().json(json!({"status":"ok"})))
}
