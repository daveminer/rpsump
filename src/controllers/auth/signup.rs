use actix_web::{post, web, web::Data, HttpRequest, HttpResponse, Result};
use chrono::Utc;
use serde::Deserialize;
use validator::Validate;

use crate::auth::password::Password;
use crate::config::Settings;
use crate::controllers::auth::helpers::{error_response, ip_address, validate_password_strength};
use crate::repository::Repo;
use crate::util::ApiResponse;

#[derive(Debug, Deserialize, Validate)]
pub struct SignupParams {
    #[validate(email)]
    email: String,
    #[validate(custom(function = "validate_password_strength"))]
    password: Password,
    #[validate(must_match(
        other = "password",
        message = "Password and confirm password must match."
    ))]
    confirm_password: Password,
    /// Token from an invite email. Signup is invite-only.
    invite_token: String,
}

/// Signup requires a valid invite. This replaces the previous LAN-only nginx
/// allowlist, which could not distinguish clients once they began reaching the
/// app through the router's NAT loopback and arriving as the public address.
#[post("/signup")]
#[tracing::instrument(skip(params, repo, _settings))]
pub async fn signup(
    req: HttpRequest,
    params: web::Json<SignupParams>,
    repo: Data<Repo>,
    _settings: Data<Settings>,
) -> Result<HttpResponse> {
    // Validate params
    match &params.validate() {
        Ok(_) => (),
        Err(e) => return Ok(ApiResponse::bad_request(e.to_string())),
    };

    let signup_email = params.email.trim().to_lowercase();

    let invite = match repo.invite_by_token(params.invite_token.clone()).await {
        Ok(Some(invite)) => invite,
        Ok(None) => {
            return Ok(ApiResponse::bad_request(
                "That invitation is not valid.".to_string(),
            ))
        }
        Err(e) => return Ok(error_response(e, "Error while looking up invitation")),
    };

    if !invite.is_redeemable(Utc::now().naive_utc()) {
        return Ok(ApiResponse::bad_request(
            "That invitation has expired or has already been used.".to_string(),
        ));
    }

    // Bind the invite to its address so a forwarded link cannot be redeemed by
    // someone other than the intended recipient.
    if invite.email.trim().to_lowercase() != signup_email {
        return Ok(ApiResponse::bad_request(
            "That invitation was issued for a different email address.".to_string(),
        ));
    }

    let ip_addr: String = match ip_address(&req) {
        Ok(ip) => ip,
        Err(e) => return Ok(error_response(e, "User signup failed")),
    };

    // Hash password
    let hash = match params.password.hash() {
        Ok(password_hash) => password_hash,
        Err(_) => {
            return Ok(ApiResponse::bad_request(
                "Try a different password.".to_string(),
            ))
        }
    };

    // Create user
    let new_user = match repo.create_user(signup_email, hash, ip_addr).await {
        Ok(user) => user,
        Err(e) => return Ok(ApiResponse::bad_request(e.to_string())),
    };

    // Marks the invite used and verifies the address in one transaction. No
    // verification email is sent: the invite was delivered to this address, so
    // control of it is already established.
    match repo.redeem_invite(invite.id, new_user.id).await {
        Ok(_) => Ok(ApiResponse::ok("User created.".to_string())),
        Err(e) => Ok(error_response(e, "Could not redeem invitation")),
    }
}
