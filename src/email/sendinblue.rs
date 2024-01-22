use anyhow::{anyhow, Error};

use crate::auth::token::Token;
use crate::config::MailerConfig;
use crate::email::{Contact, Email};
use crate::models::user::User;
use crate::util::spawn_blocking_with_tracing;

pub async fn send_email_verification(
    user: User,
    mailer: MailerConfig,
    app_server_url: &str,
) -> Result<(), Error> {
    let token = Token::new_email_verification(user.id);

    //User::save_email_verification_token(user.email.clone(), token.clone(), db).await?;

    let email = new_email_verification_email(&user.email, &app_server_url, token);
    send(&mailer.auth_token, email, &mailer.server_url).await
}

pub async fn send_error_email(mailer: &MailerConfig, error_msg: &str) -> Result<(), Error> {
    let email = new_error_email(&mailer.error_contact, &error_msg);
    send(&mailer.auth_token, email, &mailer.server_url).await
}

pub async fn send_password_reset(
    user: User,
    //db: Data<dyn DbPool>,
    mailer_url: &str,
    server_url: &str,
    auth_token: &str,
) -> Result<(), Error> {
    let token = Token::new_password_reset(user.id);

    //User::save_reset_token(user.clone(), token.clone(), db).await?;
    let email = new_password_reset_email(&user.email, server_url, token)?;

    send(auth_token, email, mailer_url).await
}

async fn send(auth_token: &str, email: Email, mailer_url: &str) -> Result<(), Error> {
    let client = reqwest::Client::new();
    let auth_token_clone = auth_token.to_string();
    let mailer_clone = mailer_url.to_string();
    let body = serde_json::to_string(&email)?;

    let thread_response = spawn_blocking_with_tracing(move || {
        client
            .post(mailer_clone)
            .header("accept", "application/json")
            .header("api-key", auth_token_clone)
            .header("content-type", "application/json")
            .body(body)
            .send()
    })
    .await?;

    let response = match thread_response.await {
        Ok(response) => response,
        Err(e) => {
            tracing::error!(
                target = module_path!(),
                error = e.to_string(),
                "Failed to send email: {}",
                e
            );
            return Err(anyhow!(e));
        }
    };

    if response.status().is_success() {
        tracing::info!(target = module_path!(), "Email sent.");
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Failed to send email. Status: {}, Details: {:?}",
            response.status(),
            response.text().await?
        ))
    }
}

fn new_email_verification_email(
    contact_email: &str,
    server_url: &str,
    verification_token: Token,
) -> Email {
    let link = format!(
        "{}/auth/verify_email?token={}",
        server_url, verification_token.value
    );

    Email {
        to: vec![Contact{ email: contact_email.to_string(), name: Some(contact_email.to_string())}],
        sender: Contact {
            name: Some("RPSump".to_string()),
            email: "robo@halyard.systems".to_string()
        },
        subject: "RPSump Email Verification".to_string(),
        html_content: format!("<!DOCTYPE html><html><body><h1>Verify your email</h1><p>Follow this link to verify your email:http://{}</p></body></html>", link),
    }
}

fn new_error_email(contact_email: &str, error_msg: &str) -> Email {
    Email {
        to: vec![Contact {
            email: contact_email.to_string(),
            name: Some(contact_email.to_string()),
        }],
        sender: Contact {
            name: Some("RPSump".to_string()),
            email: "robo@halyard.systems".to_string(),
        },
        subject: "RPSump Error Report".to_string(),
        html_content: format!("<!DOCTYPE html><html><body><h1>RPSump Error Report</h1><p>An error occurred during RPSump operation:</p><p>{}</p></body></html>", error_msg)
    }
}

fn new_password_reset_email(
    contact_email: &str,
    server_url: &str,
    token: Token,
) -> Result<Email, Error> {
    let link = format!("{}/auth/reset_password?token={}", server_url, token.value);

    Ok(Email {
        to: vec![Contact{ email: contact_email.to_string(), name: None}],
        sender: Contact {
            name: Some("RPSump".to_string()),
            email: "robo@halyard.systems".to_string()
        },
        subject: "RPSump Password Reset".to_string(),
        html_content: format!("<!DOCTYPE html><html><body><h1>Reset your password</h1><p>Follow this link to reset your password:http://{}</p></body></html>", link),
        })
}
