use actix_web::{post, web, web::Data, HttpResponse, Result};
use serde::{Deserialize, Serialize};

use crate::auth::{claim::create_token, token::Token};
use crate::config::Settings;
use crate::controllers::auth::helpers::error_response;
use crate::repository::Repo;
use crate::util::ApiResponse;

#[derive(Deserialize)]
struct RefreshRequest {
    refresh_token: String,
}

#[derive(Serialize)]
struct RefreshResponse {
    token: String,
    refresh_token: String,
}

#[post("/refresh")]
#[tracing::instrument(skip(body, settings, repo))]
pub async fn refresh(
    body: web::Json<RefreshRequest>,
    settings: Data<Settings>,
    repo: Data<Repo>,
) -> Result<HttpResponse> {
    let old_token = body.into_inner().refresh_token;

    let user_id = match repo.consume_refresh_token(old_token).await {
        Ok(uid) => uid,
        Err(e) => {
            tracing::warn!(
                target = module_path!(),
                error = %e,
                "Refresh token rejected"
            );
            return Ok(ApiResponse::unauthorized(
                "Invalid or expired refresh token.".to_string(),
            ));
        }
    };

    let access_token = match create_token(
        user_id,
        settings.jwt_secret.clone(),
        settings.server.access_token_duration_minutes,
    ) {
        Ok(t) => t,
        Err(e) => return Ok(error_response(e, "Could not create access token")),
    };

    let new_refresh = Token::new_refresh_token(
        user_id,
        settings.server.refresh_token_duration_days,
    );
    let new_refresh_value = new_refresh.value.clone();

    if let Err(e) = repo.create_refresh_token(&new_refresh).await {
        return Ok(error_response(e, "Could not create refresh token"));
    }

    tracing::info!(
        target = module_path!(),
        user_id = user_id,
        "Token refreshed"
    );

    Ok(HttpResponse::Ok().json(RefreshResponse {
        token: access_token,
        refresh_token: new_refresh_value,
    }))
}
