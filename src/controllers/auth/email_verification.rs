use actix_web::{get, web, web::Data, HttpResponse, Responder, Result};
use serde::Deserialize;
use serde_json::json;

use crate::database::DbPool;
use crate::models::user::User;

#[derive(Debug, Deserialize)]
pub struct EmailVerificationParams {
    token: String,
}

#[get("/verify_email")]
async fn verify_email(
    params: web::Query<EmailVerificationParams>,
    db: Data<DbPool>,
) -> Result<impl Responder> {
    // TODO: send html response based on request content type
    match User::verify_email(params.token.clone(), db).await {
        Ok(_) => Ok(HttpResponse::Ok().body(json!({"message": "Email verified."}).to_string())),
        Err(e) => {
            Ok(HttpResponse::BadRequest().body(json!({ "message": e.to_string() }).to_string()))
        }
    }
}
