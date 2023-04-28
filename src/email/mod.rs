use anyhow::Error;
use serde::{Deserialize, Serialize};

use crate::database::DbPool;
use crate::models::user::User;

pub mod sendinblue;

const MAILER_SERVICE_URL: &str = "https://api.sendinblue.com/v3/smtp/email";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email {
    to: Vec<Contact>,
    sender: Contact,
    subject: String,
    html_content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Contact {
    email: String,
    name: Option<String>,
}

pub async fn send_email_verification(
    user: User,
    db: actix_web::web::Data<DbPool>,
    server_url: String,
    auth_token: String,
) -> Result<(), Error> {
    user.send_email_verification(db, server_url, auth_token)
        .await?;

    Ok(())
}

pub async fn send_password_reset(
    user: User,
    db: actix_web::web::Data<DbPool>,
    server_url: String,
    auth_token: String,
) -> Result<(), Error> {
    user.send_password_reset(db, server_url, auth_token).await?;

    Ok(())
}

pub async fn send(auth_token: String, email: Email) -> Result<(), Error> {
    let client = reqwest::Client::new();

    let response = client
        .post(MAILER_SERVICE_URL)
        .header("accept", "application/json")
        .header("api-key", auth_token)
        .header("content-type", "application/json")
        .body(serde_json::to_string(&email)?)
        .send()
        .await?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Failed to send email"))
    }
}
