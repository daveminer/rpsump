use actix_web::{get, web, web::Data, HttpResponse, Responder, Result};
use serde::Deserialize;
use serde_json::json;

use crate::database::DbPool;
use crate::models::user::User;

#[derive(Debug, Deserialize)]
pub struct EmailVerificationParamhttps://github.com/daveminer/rpsump/pull/10/conflict?name=src%252Fcontrollers%252Fauth%252Femail_verification.rs&ancestor_oid=402ff2d8aaddb02dc9d7de19f7f79f69f8006da0&base_oid=a6e68a4615e6abc3e5a92557707776a59965d73d&head_oid=ba24b65759b65c89f93ee418569549f1a8c25432s {
    token: String,
}

#[get("/verify_email")]
async fn verify_email(
    params: web::Query<EmailVerificationParams>,
    db: Data<DbPool>,
) -> Result<impl Responder> {
    // TODO: send html response based on request content type (and resend)
    match User::verify_email(params.token.clone(), db).await {
        Ok(_) => Ok(HttpResponse::Ok().body(json!({"message": "Email verified."}).to_string())),
        Err(e) => {
            Ok(HttpResponse::BadRequest().body(json!({ "message": e.to_string() }).to_string()))
        }
    }
}
