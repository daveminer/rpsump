use anyhow::Error;

use crate::auth::token::Token;
use crate::database::DbPool;
use crate::email::{send, Contact, Email};
use crate::models::user::User;

impl User {
    fn new_email_verification_email(
        self,
        server_url: String,
        verification_token: Token,
    ) -> Result<Email, Error> {
        let link = format!(
            "{}/verify_email?token={}",
            server_url, verification_token.value
        );

        Ok(Email {
        to: vec![Contact{ email: self.email, name: None}],
        sender: Contact {
            name: Some("RPSump".to_string()),
            email: "wtrispsn@gmail.com".to_string()
        },
        subject: "RPSump Email Verification".to_string(),
        html_content: format!("<!DOCTYPE html> <html> <body> <h1>Verify your email</h1><p>Follow this link to verify your email:\n\n{}\n\n</p>", link),
    })
    }

    fn new_password_reset_email(self, server_url: String, token: Token) -> Result<Email, Error> {
        let link = format!("{}/reset_password?token={}", server_url, token.value);

        Ok(Email {
        to: vec![Contact{ email: self.email, name: None}],
        sender: Contact {
            name: Some("RPSump".to_string()),
            email: "wtrispsn@gmail.com".to_string()
        },
        subject: "RPSump Password Reset".to_string(),
        html_content: format!("<!DOCTYPE html> <html> <body> <h1>Reset your password</h1><p>Follow this link to reset your password:\n\n{}\n\n</p>", link),
        })
    }

    pub async fn send_email_verification(
        self,
        db: actix_web::web::Data<DbPool>,
        server_url: String,
        auth_token: String,
    ) -> Result<(), Error> {
        let token = Token::new_email_verification(self.id);

        User::save_email_verification_token(self.email.clone(), token.clone(), db).await?;

        let email = Self::new_email_verification_email(self, server_url, token)?;

        send(auth_token, email).await
    }

    pub async fn send_password_reset(
        self,
        db: actix_web::web::Data<DbPool>,
        server_url: String,
        auth_token: String,
    ) -> Result<(), Error> {
        let token = Token::new_password_reset(self.id);

        User::save_reset_token(self.clone(), token.clone(), db).await?;

        let email = Self::new_password_reset_email(self.clone(), server_url, token)?;

        send(auth_token, email).await
    }
}
