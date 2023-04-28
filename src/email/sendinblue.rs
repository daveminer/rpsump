use anyhow::Error;

use crate::auth::{email_verification_token, password_reset_token};
use crate::database::DbPool;
use crate::email::{send, Contact, Email};
use crate::models::user::User;

// TODO: impl for User

fn new_email_verification_email(user: User, verification_token: String) -> Result<Email, Error> {
    //TODO: build link
    let link = format!("https://domain.tld/verify/{}", verification_token);

    Ok(Email {
        to: vec![Contact{ email: user.email, name: None}],
        sender: Contact {
            name: Some("RPSump".to_string()),
            email: "wtrispsn@gmail.com".to_string()
        },
        subject: "RPSump Email Verification".to_string(),
        html_content: format!("<!DOCTYPE html> <html> <body> <h1>Verify your email</h1><p>Follow this link to verify your email:\n\n{}\n\n</p>", link),
    })
}

fn new_password_reset_email(user: User, reset_token: String) -> Result<Email, Error> {
    // TODO: build link
    let link = format!("https://domain.tld/reset/{}", reset_token);

    Ok(Email {
        to: vec![Contact{ email: user.email, name: None}],
        sender: Contact {
            name: Some("RPSump".to_string()),
            email: "wtrispsn@gmail.com".to_string()
        },
        subject: "RPSump Password Reset".to_string(),
        html_content: format!("<!DOCTYPE html> <html> <body> <h1>Reset your password</h1><p>Follow this link to reset your password:\n\n{}\n\n</p>", link),
    })
}

//TODO: fix verif token type
pub async fn send_email_verification(
    user: User,
    db: actix_web::web::Data<DbPool>,
    auth_token: String,
) -> Result<(), Error> {
    let verification_token = email_verification_token::generate()?;

    User::save_email_verification_token(user.clone(), verification_token.0.clone(), db).await?;

    let email = new_email_verification_email(user, verification_token.0)?;

    send(auth_token, email).await
}

pub async fn send_password_reset(
    user: User,
    db: actix_web::web::Data<DbPool>,
    auth_token: String,
) -> Result<(), Error> {
    let (reset_token, reset_token_hash) = password_reset_token::generate()?;

    User::save_reset_token(user.clone(), reset_token_hash, db).await?;

    let email = new_password_reset_email(user, reset_token)?;

    send(auth_token, email).await
}
