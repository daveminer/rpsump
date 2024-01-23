use actix_web::{error, get, HttpResponse, Result};

use crate::auth::authenticated_user::AuthenticatedUser;
use crate::repository::Repo;

#[get("/sump_event")]
#[tracing::instrument(skip(repo, _req_body, _user))]
pub async fn sump_event(
    _req_body: String,
    _user: AuthenticatedUser,
    repo: Repo,
) -> Result<HttpResponse> {
    let sump_events = repo.sump_events().await.map_err(|e| {
        tracing::error!(
            target = module_path!(),
            error = e.to_string(),
            "Error while getting sump events"
        );
        Ok(error::ErrorInternalServerError("Internal server error."))
    })?;

    Ok(HttpResponse::Ok().json(sump_events))
}
