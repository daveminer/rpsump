use actix_web::HttpRequest;
use actix_web::{
    get,
    web::{Data, Query},
    HttpResponse, Result,
};

use serde::Deserialize;

use crate::auth::authenticated_user::AuthenticatedUser;
use crate::{controllers::auth::helpers::error_response, repository::Repo};

#[derive(Debug, Deserialize)]
pub struct Params {
    status: Option<String>,
}

#[get("/event")]
#[tracing::instrument(skip(req, repo, _user))]
pub async fn irrigation_event(
    req: HttpRequest,
    repo: Data<Repo>,
    _user: AuthenticatedUser,
) -> Result<HttpResponse> {
    let filter = match Query::<Params>::from_query(req.query_string()) {
        Ok(filter) => filter,
        Err(_e) => {
            return Ok(HttpResponse::BadRequest().body("invalid filter"));
        }
    };

    let irrigation_events = match repo.irrigation_events().await {
        Ok(irrigation_events) => irrigation_events,
        Err(e) => error_response(e, "Could not get irrigation events"),
    };

    Ok(HttpResponse::Ok().json(irrigation_events))
}
