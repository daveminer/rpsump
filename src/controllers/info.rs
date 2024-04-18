use actix_web::{get, HttpRequest, HttpResponse};
use actix_web::{web::Data, Result};
use serde_json::json;
use tokio::sync::Mutex;

use crate::auth::authenticated_user::AuthenticatedUser;
use crate::hydro::Hydro;

#[get("/info")]
#[tracing::instrument(skip(hydro, _user))]
async fn info(
    _req: HttpRequest,
    _user: AuthenticatedUser,
    hydro: Data<Mutex<Hydro>>,
) -> Result<HttpResponse> {
    let hydro = hydro.lock().await;

    Ok(HttpResponse::Ok().json(json!({
        "heater": hydro.heater.is_on().await,
        "poolPumpSpeed": hydro.pool_pump.speed().await,
    })))
}
