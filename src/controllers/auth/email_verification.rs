use actix_web::{
    get,
    web::{Data, Query},
    HttpResponse, Result,
};
use serde::Deserialize;
use serde_json::json;

use crate::repository::{implementation::VerifyEmailError, Repo};
use crate::util::ApiResponse;

#[derive(Debug, Deserialize)]
struct EmailVerificationParams {
    token: String,
}

#[get("/verify_email")]
#[tracing::instrument(skip(params, repo))]
pub async fn verify_email(
    params: Query<EmailVerificationParams>,
    repo: Data<Repo>,
) -> Result<HttpResponse> {
    match repo.verify_email(params.token.clone()).await {
        Ok(()) => Ok(HttpResponse::Ok().body(json!({"message": "Email verified."}).to_string())),
        Err(VerifyEmailError::DatabaseError(e) | VerifyEmailError::InternalServerError(e)) => {
            tracing::error!(
                target = module_path!(),
                error = e.to_string(),
                "Failed to verify email"
            );
            Ok(ApiResponse::internal_server_error())
        }
        Err(e) => Ok(ApiResponse::bad_request(e.to_string())),
    }
}
