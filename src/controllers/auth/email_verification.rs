use actix_web::{
    get,
    web::{Data, Query},
    HttpResponse, Result,
};
use serde::Deserialize;
use serde_json::json;

use crate::database::DbPool;
use crate::models::user::User;

#[derive(Debug, Deserialize)]
struct EmailVerificationParams {
    token: String,
}

#[get("/verify_email")]
#[tracing::instrument(skip(params, db))]
pub async fn verify_email(
    params: Query<EmailVerificationParams>,
    db: Data<DbPool>,
) -> Result<HttpResponse> {
    match User::verify_email(params.token.clone(), db).await {
        Ok(()) => Ok(HttpResponse::Ok().body(json!({"message": "Email verified."}).to_string())),
        Err(e) => {
            tracing::error!(
                target = module_path!(),
                error = e.to_string(),
                "Failed to verify email"
            );
            Ok(HttpResponse::BadRequest().body(json!({ "message": e.to_string() }).to_string()))
        }
    }
}
