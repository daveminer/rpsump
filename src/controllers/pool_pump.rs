use actix_web::{
    post,
    web::{self, Data},
    HttpResponse, Responder, Result,
};
use anyhow::Error;
use serde::Deserialize;
use serde_json::json;

use crate::{auth::authenticated_user::AuthenticatedUser, database::DbPool};
use crate::{
    controllers::ApiResponse,
    hydro::{pool_pump::PoolPumpSpeed, Hydro},
};

#[derive(Debug, Deserialize)]
pub struct PoolPumpParams {
    pub speed: PoolPumpSpeed,
}

#[post("/pool_pump")]
#[tracing::instrument(skip(_db, _user, maybe_hydro))]
pub async fn pool_pump(
    params: web::Json<PoolPumpParams>,
    _db: Data<DbPool>,
    _user: AuthenticatedUser,
    maybe_hydro: web::Data<Option<Hydro>>,
) -> Result<impl Responder> {
    if maybe_hydro.is_none() {
        return Ok(HttpResponse::Ok().body("Hydro not configured"));
    }
    let mut hydro = maybe_hydro.as_ref().clone().unwrap();

    match params.speed {
        PoolPumpSpeed::Off => {
            if let Err(e) = hydro.pool_pump.off().await {
                error_trace(&params.speed, e);
                return Ok(ApiResponse::internal_server_error());
            }
        }
        PoolPumpSpeed::Low => {
            if let Err(e) = hydro.pool_pump.on(PoolPumpSpeed::Low).await {
                error_trace(&params.speed, e);
                return Ok(ApiResponse::internal_server_error());
            }
        }
        PoolPumpSpeed::Med => {
            if let Err(e) = hydro.pool_pump.on(PoolPumpSpeed::Med).await {
                error_trace(&params.speed, e);
                return Ok(ApiResponse::internal_server_error());
            }
        }
        PoolPumpSpeed::High => {
            if let Err(e) = hydro.pool_pump.on(PoolPumpSpeed::High).await {
                error_trace(&params.speed, e);
                return Ok(ApiResponse::internal_server_error());
            }
        }
        PoolPumpSpeed::Max => {
            if let Err(e) = hydro.pool_pump.on(PoolPumpSpeed::Max).await {
                error_trace(&params.speed, e);
                return Ok(ApiResponse::internal_server_error());
            }
        }
    };

    tracing::info!("Heater status changed: {:?}", params.speed);

    Ok(HttpResponse::Ok().json(json!({"status":"ok"})))
}

fn error_trace(speed: &PoolPumpSpeed, e: Error) {
    tracing::error!("Error while setting the pool pump to {:?}: {:?}", speed, e);
}
