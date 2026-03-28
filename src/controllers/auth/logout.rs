use actix_web::{post, web, web::Data, HttpResponse, Result};
use serde::Deserialize;

use crate::repository::Repo;
use crate::util::ApiResponse;

#[derive(Deserialize)]
struct LogoutRequest {
    refresh_token: String,
}

#[post("/logout")]
#[tracing::instrument(skip(body, repo))]
pub async fn logout(
    body: web::Json<LogoutRequest>,
    repo: Data<Repo>,
) -> Result<HttpResponse> {
    let token_value = body.into_inner().refresh_token;

    // Best-effort revocation — don't leak whether the token was valid
    match repo.consume_refresh_token(token_value).await {
        Ok(user_id) => {
            tracing::info!(
                target = module_path!(),
                user_id = user_id,
                "User logged out"
            );
        }
        Err(e) => {
            tracing::warn!(
                target = module_path!(),
                error = %e,
                "Logout with invalid refresh token"
            );
        }
    }

    Ok(ApiResponse::ok("Logged out.".to_string()))
}
