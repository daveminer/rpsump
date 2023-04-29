use anyhow::Error;

use crate::auth::token::Token;
use crate::database::DbPool;
use crate::email::{Contact, Email};
use crate::models::user::User;

const MAILER_SERVICE_URL: &str = "https://api.sendinblue.com/v3/smtp/email";

pub async fn send_email_verification(
    user: User,
    db: actix_web::web::Data<DbPool>,
    server_url: String,
    auth_token: String,
) -> Result<(), Error> {
    let token = Token::new_email_verification(user.id);

    User::save_email_verification_token(user.email.clone(), token.clone(), db).await?;

    let email = new_email_verification_email(user.email, server_url, token)?;

    send(auth_token, email).await
}

pub async fn send_password_reset(
    user: User,
    db: actix_web::web::Data<DbPool>,
    server_url: String,
    auth_token: String,
) -> Result<(), Error> {
    let token = Token::new_password_reset(user.id);

    User::save_reset_token(user.clone(), token.clone(), db).await?;

    let email = new_password_reset_email(user.email, server_url, token)?;

    send(auth_token, email).await
}

async fn send(auth_token: String, email: Email) -> Result<(), Error> {
    let client = reqwest::Client::new();

    let body = serde_json::to_string(&email)?;

    let response = client
        .post(MAILER_SERVICE_URL)
        .header("accept", "application/json")
        .header("api-key", auth_token)
        .header("content-type", "application/json")
        .body(body)
        .send()
        .await?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Failed to send email"))
    }
}

fn new_email_verification_email(
    contact_email: String,
    server_url: String,
    verification_token: Token,
) -> Result<Email, Error> {
    let link = format!(
        "{}/auth/verify_email?token={}",
        server_url, verification_token.value
    );

    Ok(Email {
        to: vec![Contact{ email: contact_email.clone(), name: Some(contact_email)}],
        sender: Contact {
            name: Some("RPSump".to_string()),
            email: "dave.miner@live.com".to_string()
        },
        subject: "RPSump Email Verification".to_string(),
        html_content: format!("<!DOCTYPE html> <html> <body> <h1>Verify your email</h1><p>Follow this link to verify your email:\n\nhttp://{}\n\n</p></body></html>", link),
    })
}

fn new_password_reset_email(
    contact_email: String,
    server_url: String,
    token: Token,
) -> Result<Email, Error> {
    let link = format!("{}/auth/reset_password?token={}", server_url, token.value);

    Ok(Email {
        to: vec![Contact{ email: contact_email, name: None}],
        sender: Contact {
            name: Some("RPSump".to_string()),
            email: "dave.miner@live.com".to_string()
        },
        subject: "RPSump Password Reset".to_string(),
        html_content: format!("<!DOCTYPE html><html><body><h1>Reset your password</h1><p>Follow this link to reset your password:\n\nhttp://{}\n\n</p></body></html>", link),
        })
}
