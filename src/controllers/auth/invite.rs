use actix_web::{post, web, web::Data, HttpResponse, Result};
use serde::Deserialize;
use validator::Validate;

use crate::auth::authenticated_user::AuthenticatedUser;
use crate::config::Settings;
use crate::controllers::auth::helpers::error_response;
use crate::email::send_invite;
use crate::repository::models::user::UserFilter;
use crate::repository::Repo;
use crate::util::ApiResponse;

#[derive(Debug, Deserialize, Validate)]
pub struct InviteParams {
    #[validate(email)]
    email: String,
}

/// Any existing user may invite another. Account creation is rare and the user
/// set is small, so a role column would add a bootstrapping problem without
/// buying anything.
#[post("/invite")]
#[tracing::instrument(skip(params, repo, settings))]
pub async fn invite(
    params: web::Json<InviteParams>,
    repo: Data<Repo>,
    settings: Data<Settings>,
    user: AuthenticatedUser,
) -> Result<HttpResponse> {
    if let Err(e) = params.validate() {
        return Ok(ApiResponse::bad_request(e.to_string()));
    }

    let invitee_email = params.email.trim().to_lowercase();

    // Refuse to invite an address that already has an account. Returning this
    // plainly is fine: the caller is authenticated and can see the user list
    // through the app anyway.
    let existing = repo
        .users(UserFilter {
            email: Some(invitee_email.clone()),
            ..Default::default()
        })
        .await;

    match existing {
        Ok(users) if !users.is_empty() => {
            return Ok(ApiResponse::bad_request(
                "An account with that email already exists.".to_string(),
            ));
        }
        Ok(_) => (),
        Err(e) => return Ok(error_response(e, "Could not check for an existing user")),
    }

    let inviter = match repo
        .users(UserFilter {
            id: Some(user.id),
            ..Default::default()
        })
        .await
    {
        Ok(users) => match users.into_iter().next() {
            Some(u) => u,
            None => return Ok(ApiResponse::internal_server_error()),
        },
        Err(e) => return Ok(error_response(e, "Could not load the inviting user")),
    };

    let invite = match repo.create_invite(invitee_email.clone(), user.id).await {
        Ok(invite) => invite,
        Err(e) => return Ok(error_response(e, "Could not create invite")),
    };

    match send_invite(
        &invitee_email,
        &inviter.email,
        &invite.token,
        &settings.server.public_app_url,
        &settings.mailer,
    )
    .await
    {
        Ok(_) => Ok(ApiResponse::ok("Invite sent.".to_string())),
        Err(e) => Ok(error_response(e, "Invite created but could not be sent")),
    }
}
