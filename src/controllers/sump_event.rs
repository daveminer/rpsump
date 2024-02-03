use actix_web::web::Data;
use actix_web::{get, HttpResponse, Result};

use crate::auth::authenticated_user::AuthenticatedUser;
use crate::controllers::auth::helpers::error_response;
use crate::repository::Repo;

#[get("/sump_event")]
#[tracing::instrument(skip(repo, _req_body, _user))]
pub async fn sump_event(
    _req_body: String,
    _user: AuthenticatedUser,
    repo: Data<Repo>,
) -> Result<HttpResponse> {
    let sump_events = match repo.sump_events().await {
        Ok(sump_events) => sump_events,
        Err(e) => {
            return Ok(error_response(e, "Could not get sump events"));
        }
    };

    Ok(HttpResponse::Ok().json(sump_events))
}
